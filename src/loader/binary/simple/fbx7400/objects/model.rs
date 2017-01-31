//! `Model` nodes.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Model` nodes.
///
/// `FbxNode` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `MultiLayer`.
    pub multi_layer: Option<i32>,
    /// `MultiTake`.
    pub multi_take: Option<i32>,
    /// `Shading`.
    pub shading: bool,
    /// `Culling`.
    pub culling: CullingType,
    /// Properties.
    pub properties: Properties70,
}

impl Model {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut multi_layer = None;
        let mut multi_take = None;
        let mut shading = None;
        let mut culling = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, ModelChildAttrs::load);
            match node_type {
                ModelChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                ModelChildAttrs::MultiLayer(v) => {
                    multi_layer = Some(v);
                    parser.skip_current_node()?;
                },
                ModelChildAttrs::MultiTake(v) => {
                    multi_take = Some(v);
                    parser.skip_current_node()?;
                },
                ModelChildAttrs::Shading(v) => {
                    shading = Some(v);
                    parser.skip_current_node()?;
                },
                ModelChildAttrs::Culling(v) => {
                    use std::str::FromStr;

                    let v = CullingType::from_str(&v).ok();
                    if v.is_none() {
                        return Err(Error::InvalidAttribute("Culling".to_owned()));
                    }
                    culling = v;
                    parser.skip_current_node()?;
                },
                ModelChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Model {
            id: obj_props.id,
            version: ensure_node_exists! { version,
                format!("Model (class=`{}`, subclass=`{}`)", obj_props.class, obj_props.subclass)},
            multi_layer: multi_layer,
            multi_take: multi_take,
            shading: ensure_node_exists! { shading,
                format!("Model (class=`{}`, subclass=`{}`)", obj_props.class, obj_props.subclass)},
            culling: ensure_node_exists! { culling,
                format!("Model (class=`{}`, subclass=`{}`)", obj_props.class, obj_props.subclass)},
            properties: ensure_node_exists! { properties,
                format!("Model (class=`{}`, subclass=`{}`)", obj_props.class, obj_props.subclass)},
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Model")
            .and_then(|ot| ot.property_template.get("FbxNode"))
    }
}


child_attr_loader! { ModelChildAttrs {
    "Version" => Version(i32),
    "Properties70" => Properties70,
    "MultiLayer" => MultiLayer(i32),
    "MultiTake" => MultiTake(i32),
    "Shading" => Shading(bool),
    "Culling" => Culling(String),
}}


/// `FbxNode::ECullingType` (hidden) of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullingType {
    /// `ECullingType::eCullingOff`.
    Off,
    /// `ECullingType::eCullingOnCCW`.
    OnCcw,
    /// `ECullingType::eCullingOnCW`.
    OnCw,
}

impl ::std::str::FromStr for CullingType {
    type Err = ();

    /// Creates a new `CullingType` from the string (used in FBX).
    fn from_str(v: &str) -> ::std::result::Result<Self, Self::Err> {
        match v {
            "CullingOff" => Ok(CullingType::Off),
            "CullingOnCCW" => Ok(CullingType::OnCcw),
            "CullingOnCW" => Ok(CullingType::OnCw),
            _ => Err(()),
        }
    }
}
