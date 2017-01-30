//! `NodeAttribute` nodes.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `NodeAttribute` node with class=`NodeAttribute`, subclass=`Null`.
///
/// `FbxNull` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct NodeAttributeNull {
    /// Object id.
    pub id: i64,
    /// `TypeFlags`.
    ///
    /// Always `Null`?
    pub type_flags: String,
    /// Properties.
    pub properties: Properties70,
}

impl NodeAttributeNull {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: ObjectProperties
    ) -> Result<Self> {
        let mut type_flags = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, NodeAttributeNullChildAttrs::load);
            match node_type {
                NodeAttributeNullChildAttrs::TypeFlags(v) => {
                    type_flags = Some(v);
                    parser.skip_current_node()?;
                },
                NodeAttributeNullChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(NodeAttributeNull {
            id: obj_props.id,
            type_flags: ensure_node_exists!(type_flags,
                                            node_msg!(NodeAttribute, NodeAttribute, Null)),
            properties: ensure_node_exists!(properties,
                                            node_msg!(NodeAttribute, NodeAttribute, Null)),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("NodeAttribute")
            .and_then(|ot| ot.property_template.get("FbxNull"))
    }

    /// `Color`.
    pub fn get_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "Color").map(|v| *v.value())
    }

    /// `Look`.
    pub fn get_look(&self, defaults: &Option<&Properties70>) -> Option<NullNodeLook> {
        get_property!(self.properties, defaults, values_i64, "Look")
            .and_then(|v| NullNodeLook::from_i64(*v.value()))
    }

    /// `Size`.
    pub fn get_size(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "Size").map(|v| *v.value())
    }
}


child_attr_loader! { NodeAttributeNullChildAttrs {
    "TypeFlags" => TypeFlags(String),
    "Properties70" => Properties70,
}}


/// Null node look types.
///
/// `FbxNull::ELook` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NullNodeLook {
    /// `ELook::eNone`.
    None = 0,
    /// `ELook::eCross`.
    Cross = 1,
}

impl NullNodeLook {
    /// CReates an enum value from the given value.
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            v if v == NullNodeLook::None as i64 => Some(NullNodeLook::None),
            v if v == NullNodeLook::Cross as i64 => Some(NullNodeLook::Cross),
            _ => None,
        }
    }
}
