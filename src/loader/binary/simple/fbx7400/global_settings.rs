//! `GlobalSettings` node and its children.

use parser::binary::{Parser, ParserSource, Event, Attributes};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::Properties70;


/// `GlobalSettings` node.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSettings {
    /// Version of the node.
    pub version: i32,
    /// Properties.
    pub properties: Properties70,
}

impl GlobalSettings {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut version = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, GlobalSettingsChildAttrs::load);
            match node_type {
                GlobalSettingsChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                GlobalSettingsChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(GlobalSettings {
            version: ensure_node_exists!(version, "Definitions"),
            properties: ensure_node_exists!(properties, "Definitions"),
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GlobalSettingsChildAttrs {
    Version(i32),
    Properties70,
}

impl GlobalSettingsChildAttrs {
    /// Creates an `ObjectTypeChildAttrs` from the given node name.
    pub fn load<R: ParserSource>(name: &str, mut attrs: Attributes<R>) -> Result<Self> {
        use parser::binary::utils::AttributeValues;

        match name {
            "Version" => {
                <i32>::from_attributes(&mut attrs)
                    ?
                    .ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
                    .map(GlobalSettingsChildAttrs::Version)
            },
            "Properties70" => Ok(GlobalSettingsChildAttrs::Properties70),
            _ => Err(Error::UnexpectedNode(name.to_owned())),
        }
    }
}
