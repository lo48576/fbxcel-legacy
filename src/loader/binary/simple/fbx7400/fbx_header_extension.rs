//! `Definitions` node and its children.

use parser::binary::{Parser, ParserSource, Event, Attributes};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::separate_name_class;
use loader::binary::simple::fbx7400::Properties70;


/// `FBXHeaderExtension` node.
#[derive(Debug, Clone, PartialEq)]
pub struct FbxHeaderExtension {
    /// Version of the node.
    pub fbx_header_version: i32,
    /// Version of the FBX.
    pub fbx_version: i32,
    /// Encryption type.
    pub encryption_type: i32,
    /// Creation time stamp.
    pub creation_timestamp: CreationTimeStamp,
    /// Creator.
    pub creator: String,
    /// Scene info.
    pub scene_info: SceneInfo,
}

impl FbxHeaderExtension {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut fbx_header_version = None;
        let mut fbx_version = None;
        let mut encryption_type = None;
        let mut creation_timestamp = None;
        let mut creator = None;
        let mut scene_info = None;

        loop {
            let node_type = try_get_node_attrs!(parser, FbxHeaderExtensionChildAttrs::load);
            match node_type {
                FbxHeaderExtensionChildAttrs::FbxHeaderVersion(v) => {
                    fbx_header_version = Some(v);
                    parser.skip_current_node()?;
                },
                FbxHeaderExtensionChildAttrs::FbxVersion(v) => {
                    fbx_version = Some(v);
                    parser.skip_current_node()?;
                },
                FbxHeaderExtensionChildAttrs::EncryptionType(v) => {
                    encryption_type = Some(v);
                    parser.skip_current_node()?;
                },
                FbxHeaderExtensionChildAttrs::CreationTimeStamp => {
                    creation_timestamp = Some(CreationTimeStamp::load(parser.subtree_parser())?);
                },
                FbxHeaderExtensionChildAttrs::Creator(v) => {
                    creator = Some(v);
                    parser.skip_current_node()?;
                },
                FbxHeaderExtensionChildAttrs::SceneInfo(attrs) => {
                    scene_info = Some(SceneInfo::load(parser.subtree_parser(), attrs)?);
                },
            }
        }
        Ok(FbxHeaderExtension {
            fbx_header_version: ensure_node_exists!(fbx_header_version, "FbxHeaderExtension"),
            fbx_version: ensure_node_exists!(fbx_version, "FbxHeaderExtension"),
            encryption_type: ensure_node_exists!(encryption_type, "FbxHeaderExtension"),
            creation_timestamp: ensure_node_exists!(creation_timestamp, "FbxHeaderExtension"),
            creator: ensure_node_exists!(creator, "FbxHeaderExtension"),
            scene_info: ensure_node_exists!(scene_info, "FbxHeaderExtension"),
        })
    }
}


child_attr_loader! { FbxHeaderExtensionChildAttrs {
    "FBXHeaderVersion" => FbxHeaderVersion(i32),
    "FBXVersion" => FbxVersion(i32),
    "EncryptionType" => EncryptionType(i32),
    "CreationTimeStamp" => CreationTimeStamp,
    "Creator" => Creator(String),
    "SceneInfo" => SceneInfo((String, String)),
}}


/// Creation time stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreationTimeStamp {
    /// Version.
    pub version: i32,
    /// Year.
    pub year: i32,
    /// Month.
    pub month: i32,
    /// Day.
    pub day: i32,
    /// Hour.
    pub hour: i32,
    /// Minute.
    pub minute: i32,
    /// Second.
    pub second: i32,
    /// Millisecond.
    pub millisecond: i32,
}

impl CreationTimeStamp {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut version = None;
        let mut year = None;
        let mut month = None;
        let mut day = None;
        let mut hour = None;
        let mut minute = None;
        let mut second = None;
        let mut millisecond = None;

        loop {
            let node_type = try_get_node_attrs!(parser, CreationTimeStampChildAttrs::load);
            match node_type {
                CreationTimeStampChildAttrs::Version(v) => {
                    version = Some(v);
                },
                CreationTimeStampChildAttrs::Year(v) => {
                    year = Some(v);
                },
                CreationTimeStampChildAttrs::Month(v) => {
                    month = Some(v);
                },
                CreationTimeStampChildAttrs::Day(v) => {
                    day = Some(v);
                },
                CreationTimeStampChildAttrs::Hour(v) => {
                    hour = Some(v);
                },
                CreationTimeStampChildAttrs::Minute(v) => {
                    minute = Some(v);
                },
                CreationTimeStampChildAttrs::Second(v) => {
                    second = Some(v);
                },
                CreationTimeStampChildAttrs::Millisecond(v) => {
                    millisecond = Some(v);
                },
            }
            parser.skip_current_node()?;
        }
        Ok(CreationTimeStamp {
            version: ensure_node_exists!(version, "CreationTimeStamp"),
            year: ensure_node_exists!(year, "CreationTimeStamp"),
            month: ensure_node_exists!(month, "CreationTimeStamp"),
            day: ensure_node_exists!(day, "CreationTimeStamp"),
            hour: ensure_node_exists!(hour, "CreationTimeStamp"),
            minute: ensure_node_exists!(minute, "CreationTimeStamp"),
            second: ensure_node_exists!(second, "CreationTimeStamp"),
            millisecond: ensure_node_exists!(millisecond, "CreationTimeStamp"),
        })
    }
}


child_attr_loader! { CreationTimeStampChildAttrs {
    "Version" => Version(i32),
    "Year" => Year(i32),
    "Month" => Month(i32),
    "Day" => Day(i32),
    "Hour" => Hour(i32),
    "Minute" => Minute(i32),
    "Second" => Second(i32),
    "Millisecond" => Millisecond(i32),
}}


/// Scene info.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneInfo {
    /// Object name?
    pub name: String,
    /// Object class?
    ///
    /// This is usually `SceneInfo`?
    pub class: String,
    /// Object subclass?
    ///
    /// This is usually `UserData`?
    pub subclass: String,
    /// Type.
    ///
    /// This is usually `UserData`?
    pub type_: String,
    /// Version.
    pub version: i32,
    /// Metadata about the FBX data.
    pub metadata: MetaData,
    /// Properties about FBX file and data.
    pub properties: Properties70,
}

impl SceneInfo {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        attrs: (String, String)
    ) -> Result<Self> {
        let mut type_ = None;
        let mut version = None;
        let mut metadata = None;
        let mut properties = None;

        // Attrs.
        let (name, class) = separate_name_class(&attrs.0).map(|(n, c)| (n.into(), c.into()))
            .ok_or_else(|| Error::InvalidAttribute("SceneInfo".to_owned()))?;
        let subclass = attrs.1;


        loop {
            let node_type = try_get_node_attrs!(parser, SceneInfoChildAttrs::load);
            match node_type {
                SceneInfoChildAttrs::Type(v) => {
                    type_ = Some(v);
                    parser.skip_current_node()?;
                },
                SceneInfoChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                SceneInfoChildAttrs::MetaData => {
                    metadata = Some(MetaData::load(parser.subtree_parser())?);
                },
                SceneInfoChildAttrs::Properties => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(SceneInfo {
            name: name,
            class: class,
            subclass: subclass,
            type_: ensure_node_exists!(type_, "SceneInfo"),
            version: ensure_node_exists!(version, "SceneInfo"),
            metadata: ensure_node_exists!(metadata, "SceneInfo"),
            properties: ensure_node_exists!(properties, "SceneInfo"),
        })
    }
}


child_attr_loader! { SceneInfoChildAttrs {
    "Type" => Type(String),
    "Version" => Version(i32),
    "MetaData" => MetaData,
    "Properties70" => Properties,
}}


/// FBX metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetaData {
    /// Version.
    pub version: i32,
    /// Title.
    pub title: String,
    /// Subject.
    pub subject: String,
    /// Author.
    pub author: String,
    /// Keywords.
    pub keywords: String,
    /// Revision.
    pub revision: String,
    /// Comment.
    pub comment: String,
}

impl MetaData {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut version = None;
        let mut title = None;
        let mut subject = None;
        let mut author = None;
        let mut keywords = None;
        let mut revision = None;
        let mut comment = None;

        loop {
            let node_type = try_get_node_attrs!(parser, MetaDataChildAttrs::load);
            match node_type {
                MetaDataChildAttrs::Version(v) => {
                    version = Some(v);
                },
                MetaDataChildAttrs::Title(v) => {
                    title = Some(v);
                },
                MetaDataChildAttrs::Subject(v) => {
                    subject = Some(v);
                },
                MetaDataChildAttrs::Author(v) => {
                    author = Some(v);
                },
                MetaDataChildAttrs::Keywords(v) => {
                    keywords = Some(v);
                },
                MetaDataChildAttrs::Revision(v) => {
                    revision = Some(v);
                },
                MetaDataChildAttrs::Comment(v) => {
                    comment = Some(v);
                },
            }
            parser.skip_current_node()?;
        }
        Ok(MetaData {
            version: ensure_node_exists!(version, "MetaData"),
            title: ensure_node_exists!(title, "MetaData"),
            subject: ensure_node_exists!(subject, "MetaData"),
            author: ensure_node_exists!(author, "MetaData"),
            keywords: ensure_node_exists!(keywords, "MetaData"),
            revision: ensure_node_exists!(revision, "MetaData"),
            comment: ensure_node_exists!(comment, "MetaData"),
        })
    }
}


child_attr_loader! { MetaDataChildAttrs {
    "Version" => Version(i32),
    "Title" => Title(String),
    "Subject" => Subject(String),
    "Author" => Author(String),
    "Keywords" => Keywords(String),
    "Revision" => Revision(String),
    "Comment" => Comment(String),
}}
