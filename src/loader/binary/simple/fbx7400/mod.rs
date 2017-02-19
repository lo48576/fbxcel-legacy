//! Simple FBX 7.4 binary loader.

use parser::binary::{Parser, ParserSource, FbxFooter, Event, Attributes};
use loader::binary::simple::{Result, Error, GenericNode};
pub use self::connections::{Connections, Connection};
pub use self::definitions::{Definitions, ObjectType};
pub use self::fbx_header_extension::{FbxHeaderExtension, CreationTimeStamp, SceneInfo};
pub use self::global_settings::GlobalSettings;
pub use self::objects::{LoadObjects7400, ObjectProperties};
pub use self::properties70::{Properties70, PropertyMap, PropertyValue};
pub use self::takes::{Takes, Take};


/// Tries to load the node attributes for parsing a child node.
///
/// The type of `$parser` should be `P: Parser<R> where R: ParserSource`, and
/// the type of `$load_attr` should be `F: FnOnce(&str, &mut Attributes) -> Result<T, _>`.
///
/// This will returns from the parent function on errors.
macro_rules! try_get_node_attrs {
    ($parser:expr, $load_attr:expr) => {{
        use $crate::parser::binary::Event;
        match $parser.next_event()? {
            Event::StartNode(info) => $load_attr(info.name, info.attributes)?,
            Event::EndNode => break,
            ev => panic!("Unexpected node event: {:?}", ev),
        }
    }}
}


/// Unwraps `$node_op` or returns `Error::MissingNode` error.
macro_rules! ensure_node_exists {
    ($node_opt:expr, $parent:expr, $child:expr) => {
        $node_opt.ok_or_else(|| {
            $crate::loader::binary::simple::Error::missing_node($parent, $child)
        })?
    };
}


macro_rules! child_attr_loader {
    (@load $enum_name:ident; $_name:ident; $_attrs:ident; $variant:ident($content:ty);
        => $load:block $(=> $_rest_load:block)*) => {
        $load
    };
    (@load $enum_name:ident; $name:ident; $attrs:ident; $variant:ident($content:ty);) => {
        <$content>::from_attributes(&mut $attrs)
            ?
            .ok_or_else(|| Error::InvalidAttribute($name.to_owned()))
            .map($enum_name::$variant)
    };
    (@load $enum_name:ident; $name:ident; $attrs:ident; $variant:ident;) => {
        Ok($enum_name::$variant)
    };
    ($enum_name:ident {
        $($node_name:expr => $variant:ident$(($content:ty))* $(=> $load:block)*),*,
    }) => {
        #[derive(Debug)]
        enum $enum_name {
            $($variant$(($content))*),*,
        }
        impl $enum_name {
            pub fn load<R: ParserSource>(name: &str, mut attrs: Attributes<R>)
                -> $crate::loader::binary::simple::Result<Self> {
                use parser::binary::utils::AttributeValues;

                match name {
                    $($node_name => child_attr_loader!{
                        @load $enum_name; name; attrs; $variant$(($content))*; $(=> $load)*
                    }),*,
                    _ => Err(Error::UnexpectedNode(name.to_owned())),
                }
            }
        }
    };
}


pub mod connections;
pub mod definitions;
pub mod fbx_header_extension;
pub mod global_settings;
pub mod objects;
pub mod properties70;
pub mod takes;


/// FBX 7.4 or later.
#[derive(Debug, Clone, PartialEq)]
pub struct Fbx7400<O: LoadObjects7400> {
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
    pub objects: O::Objects,
    /// `Connections`.
    pub connections: Connections,
    /// `Takes`.
    pub takes: Option<Takes>,
    /// FBX footer.
    pub footer: Option<FbxFooter>,
}

impl<O: LoadObjects7400> Fbx7400<O> {
    /// Loads FBX 7400 (or later) structure from the given parser.
    pub fn load_from_parser<R: ParserSource, P: Parser<R>>(
        version: u32,
        mut parser: P,
        objs_loader: O
    ) -> Result<Self> {
        info!("FBX version: {}, loading in FBX 7400 mode", version);

        let mut objs_loader = Some(objs_loader);
        let footer;
        let mut fbx_header_extension = None;
        let mut file_id = None;
        let mut creation_time = None;
        let mut creator = None;
        let mut global_settings = None;
        let mut documents = None;
        let mut references = None;
        let mut definitions = None;
        let mut objects_and_before = None;
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
                    if let Some(objs_loader) = objs_loader.take() {
                        let nodes_before_objects = NodesBeforeObjects {
                            version: version,
                            fbx_header_extension: ensure_node_exists!(fbx_header_extension.take(),
                                                                      "(root)",
                                                                      "FBXHeaderExtension"),
                            file_id: ensure_node_exists!(file_id.take(), "(root)", "FileId"),
                            creation_time: ensure_node_exists!(creation_time.take(),
                                                               "(root)",
                                                               "CreationTime"),
                            creator: ensure_node_exists!(creator.take(), "(root)", "Creator"),
                            global_settings: ensure_node_exists!(global_settings.take(),
                                                                 "(root)",
                                                                 "GlobalSettings"),
                            documents: ensure_node_exists!(documents.take(), "(root)", "Documents"),
                            references: ensure_node_exists!(references.take(),
                                                            "(root)",
                                                            "References"),
                            definitions: ensure_node_exists!(definitions.take(),
                                                             "(root)",
                                                             "Definitions"),
                        };
                        let objects = load_objects(parser.subtree_parser(),
                                                   objs_loader,
                                                   &nodes_before_objects)?;
                        objects_and_before = Some((objects, nodes_before_objects));
                    } else {
                        warn!("Multiple `Objects` node found, ignoring.");
                    }
                },
                NodeType::Connections => {
                    connections = Some(Connections::load(parser.subtree_parser())?);
                },
                NodeType::Takes => {
                    takes = Some(Takes::load(parser.subtree_parser())?);
                },
            }
        }

        let (objects, nodes_before_objects) =
            ensure_node_exists!(objects_and_before, "(root)", "Objects");

        Ok(Fbx7400 {
            version: version,
            fbx_header_extension: nodes_before_objects.fbx_header_extension,
            file_id: nodes_before_objects.file_id,
            creation_time: nodes_before_objects.creation_time,
            creator: nodes_before_objects.creator,
            global_settings: nodes_before_objects.global_settings,
            documents: nodes_before_objects.documents,
            references: nodes_before_objects.references,
            definitions: nodes_before_objects.definitions,
            objects: objects,
            connections: ensure_node_exists!(connections, "(root)", "Connections"),
            takes: takes,
            footer: footer,
        })
    }
}


/// Toplevel nodes before `Objects`.
///
/// These nodes would be referred by objects loader.
#[derive(Debug, Clone, PartialEq)]
pub struct NodesBeforeObjects {
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


/// Returns `Option<(name: &'a str, class: &'a str)>`
pub fn separate_name_class(name_class: &str) -> Option<(&str, &str)> {
    name_class.find("\u{0}\u{1}")
        .map(|sep_pos| (&name_class[0..sep_pos], &name_class[sep_pos + 2..]))
}


/// Loads node contents from the parser.
fn load_objects<R: ParserSource, P: Parser<R>, O: LoadObjects7400>(
    mut parser: P,
    mut objs_loader: O,
    nodes_before_objects: &NodesBeforeObjects
) -> Result<O::Objects> {
    loop {
        let props = try_get_node_attrs!(parser, ObjectProperties::load);
        let mut sub_parser = parser.subtree_parser();
        objs_loader.load(props, &mut sub_parser, nodes_before_objects)?;
        sub_parser.skip_to_end()?;
    }
    objs_loader.build()
}
