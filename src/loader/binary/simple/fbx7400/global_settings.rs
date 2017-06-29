//! `GlobalSettings` node and its children.

use parser::binary::{Parser, ParserSource, Attributes};
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
    pub fn load<R, P>(mut parser: P) -> Result<Self>
        where R: ParserSource,
              P: Parser<R>
    {
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
               version: ensure_node_exists!(version, "Definitions", "Version"),
               properties: ensure_node_exists!(properties, "Definitions", "Properties70"),
           })
    }
}


child_attr_loader! { GlobalSettingsChildAttrs {
    "Version" => Version(i32),
    "Properties70" => Properties70,
}}
