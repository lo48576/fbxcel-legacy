//! Simple FBX 7.4 binary loader.

use parser::binary::{Parser, ParserSource, FbxFooter, Event, Attributes};
use loader::binary::simple::{Result, Error, GenericNode};
pub use self::connections::{Connections, Connection};
pub use self::definitions::{Definitions, ObjectType};
pub use self::properties70::{Properties70, PropertyMap, PropertyValue};


/// Tries to load the node attributes for parsing a child node.
///
/// The type of `$parser` should be `P: Parser<R> where R: ParserSource`, and
/// the type of `$load_attr` should be `F: FnOnce(&str, &mut Attributes) -> Result<T, _>`.
///
/// This will returns from the parent function on errors.
macro_rules! try_get_node_attrs {
    ($parser:expr, $load_attr:expr) => {
        match $parser.next_event()? {
            Event::StartNode(info) => $load_attr(info.name, info.attributes)?,
            Event::EndNode => break,
            ev => panic!("Unexpected node event: {:?}", ev),
        }
    }
}


pub mod connections;
pub mod definitions;
pub mod properties70;


/// FBX 7.4 or later.
#[derive(Debug, Clone, PartialEq)]
pub struct Fbx7400 {
    /// FBX version.
    pub version: u32,
    /// `FBXHeaderExtension`.
    pub fbx_header_extension: FbxHeaderExtension,
    /// `FileId`.
    pub file_id: FileId,
    /// `CreationTime`.
    pub creation_time: CreationTime,
    /// `Creator`.
    pub creator: Creator,
    /// `References`.
    pub references: References,
    /// `GlobalSettings`.
    pub global_settings: GlobalSettings,
    /// `Documents`.
    pub documents: Documents,
    /// `Definitions`.
    pub definitions: Definitions,
    /// `Objects`.
    pub objects: Objects,
    /// `Connections`.
    pub connections: Connections,
    /// `Takes`.
    pub takes: Option<Takes>,
    /// FBX footer.
    pub footer: Option<FbxFooter>,
}

impl Fbx7400 {
    /// Loads FBX 7400 (or later) structure from the given parser.
    pub fn load_from_parser<R: ParserSource, P: Parser<R>>(
        version: u32,
        mut parser: P
    ) -> Result<Self> {
        info!("FBX version: {}, loading in FBX 7400 mode", version);

        let footer;
        let mut fbx_header_extension = None;
        let mut file_id = None;
        let mut creation_time = None;
        let mut creator = None;
        let mut global_settings = None;
        let mut documents = None;
        let mut references = None;
        let mut definitions = None;
        let mut objects = None;
        let mut connections = None;
        let mut takes = None;
        loop {
            let node_type = match parser.next_event()? {
                Event::StartFbx(_) |
                Event::EndNode => unreachable!(),
                Event::EndFbx(f) => {
                    footer = f.ok();
                    break;
                },
                Event::StartNode(info) => NodeType::load(info.name, info.attributes)?,
            };
            debug!("node_type: {:?}", node_type);
            match node_type {
                NodeType::FbxHeaderExtension => {
                    fbx_header_extension = Some(FbxHeaderExtension::load(parser.subtree_parser())?);
                },
                NodeType::FileId(attrs) => {
                    file_id = Some(FileId::load(parser.subtree_parser(), attrs)?);
                },
                NodeType::CreationTime(attrs) => {
                    creation_time = Some(CreationTime::load(parser.subtree_parser(), attrs)?);
                },
                NodeType::Creator(attrs) => {
                    creator = Some(Creator::load(parser.subtree_parser(), attrs)?);
                },
                NodeType::GlobalSettings => {
                    global_settings = Some(GlobalSettings::load(parser.subtree_parser())?);
                },
                NodeType::Documents => {
                    documents = Some(Documents::load(parser.subtree_parser())?);
                },
                NodeType::References => {
                    references = Some(References::load(parser.subtree_parser())?);
                },
                NodeType::Definitions => {
                    definitions = Some(Definitions::load(parser.subtree_parser())?);
                },
                NodeType::Objects => {
                    objects = Some(Objects::load(parser.subtree_parser())?);
                },
                NodeType::Connections => {
                    connections = Some(Connections::load(parser.subtree_parser())?);
                },
                NodeType::Takes => {
                    takes = Some(Takes::load(parser.subtree_parser())?);
                },
            }
        }

        Ok(Fbx7400 {
            version: version,
            fbx_header_extension: fbx_header_extension.ok_or_else(|| Error::MissingNode("FBXHeaderExtension".to_owned()))?,
            file_id: file_id.ok_or_else(|| Error::MissingNode("FileId".to_owned()))?,
            creation_time: creation_time.ok_or_else(|| Error::MissingNode("CreationTime".to_owned()))?,
            creator: creator.ok_or_else(|| Error::MissingNode("Creator".to_owned()))?,
            global_settings: global_settings.ok_or_else(|| Error::MissingNode("GlobalSettings".to_owned()))?,
            documents: documents.ok_or_else(|| Error::MissingNode("Documents".to_owned()))?,
            references: references.ok_or_else(|| Error::MissingNode("References".to_owned()))?,
            definitions: definitions.ok_or_else(|| Error::MissingNode("Definitions".to_owned()))?,
            objects: objects.ok_or_else(|| Error::MissingNode("Objects".to_owned()))?,
            connections: connections.ok_or_else(|| Error::MissingNode("Connections".to_owned()))?,
            takes: takes,
            footer: footer,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NodeType {
    FbxHeaderExtension,
    FileId(Vec<u8>),
    CreationTime(String),
    Creator(String),
    GlobalSettings,
    Documents,
    References,
    Definitions,
    Objects,
    Connections,
    Takes,
}

impl NodeType {
    /// Creates `NodeType` from the given node name.
    pub fn load<R: ParserSource>(name: &str, mut attrs: Attributes<R>) -> Result<Self> {
        use parser::binary::utils::AttributeValues;

        match name {
            "FBXHeaderExtension" => Ok(NodeType::FbxHeaderExtension),
            "FileId" => {
                <Vec<u8>>::from_attributes(&mut attrs)
                    ?
                    .ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
                    .map(NodeType::FileId)
            },
            "CreationTime" => {
                <String>::from_attributes(&mut attrs)
                    ?
                    .ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
                    .map(NodeType::CreationTime)
            },
            "Creator" => {
                <String>::from_attributes(&mut attrs)
                    ?
                    .ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
                    .map(NodeType::Creator)
            },
            "GlobalSettings" => Ok(NodeType::GlobalSettings),
            "Documents" => Ok(NodeType::Documents),
            "References" => Ok(NodeType::References),
            "Definitions" => Ok(NodeType::Definitions),
            "Objects" => Ok(NodeType::Objects),
            "Connections" => Ok(NodeType::Connections),
            "Takes" => Ok(NodeType::Takes),
            _ => Err(Error::UnexpectedNode(name.to_owned())),
        }
    }
}


/// `FBXHeaderExtension`.
#[derive(Debug, Clone, PartialEq)]
pub struct FbxHeaderExtension {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl FbxHeaderExtension {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(FbxHeaderExtension { nodes: nodes })
    }
}


/// `FileId`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub Vec<u8>);

impl FileId {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: Vec<u8>) -> Result<Self> {
        parser.skip_current_node()?;
        Ok(FileId(attrs))
    }
}


/// `CreationTime`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreationTime(pub String);

impl CreationTime {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: String) -> Result<Self> {
        parser.skip_current_node()?;
        Ok(CreationTime(attrs))
    }
}


/// `Creator`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Creator(pub String);

impl Creator {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: String) -> Result<Self> {
        parser.skip_current_node()?;
        Ok(Creator(attrs))
    }
}


/// `GlobalSettings`.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSettings {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl GlobalSettings {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(GlobalSettings { nodes: nodes })
    }
}


/// `Documents`.
#[derive(Debug, Clone, PartialEq)]
pub struct Documents {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl Documents {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(Documents { nodes: nodes })
    }
}


/// `References`.
#[derive(Debug, Clone, PartialEq)]
pub struct References {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl References {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(References { nodes: nodes })
    }
}


/// `Objects`.
#[derive(Debug, Clone, PartialEq)]
pub struct Objects {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl Objects {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(Objects { nodes: nodes })
    }
}


/// `Takes`.
#[derive(Debug, Clone, PartialEq)]
pub struct Takes {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl Takes {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(Takes { nodes: nodes })
    }
}
