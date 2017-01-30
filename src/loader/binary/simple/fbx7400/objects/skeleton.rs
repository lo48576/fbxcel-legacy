//! `Skeleton` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `NodeAttribute` node with class=`NodeAttribute`, subclass=`LimbNode`.
///
/// `FbxSkeleton` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    /// Object id.
    pub id: i64,
    /// `TypeFlags`.
    ///
    /// Always `Skeleton`?
    pub type_flags: String,
    /// Properties.
    pub properties: Properties70,
}

impl Skeleton {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: ObjectProperties
    ) -> Result<Self> {
        let mut type_flags = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, SkeletonChildAttrs::load);
            match node_type {
                SkeletonChildAttrs::TypeFlags(v) => {
                    type_flags = Some(v);
                    parser.skip_current_node()?;
                },
                SkeletonChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Skeleton {
            id: obj_props.id,
            type_flags: ensure_node_exists!(type_flags,
                                            node_msg!(NodeAttribute, NodeAttribute, LimbNode)),
            properties: ensure_node_exists!(properties,
                                            node_msg!(NodeAttribute, NodeAttribute, LimbNode)),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("NodeAttribute")
            .and_then(|ot| ot.property_template.get("FbxSkeleton"))
    }

    /// `Size`.
    pub fn get_size(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "Size").map(|v| *v.value())
    }
}


child_attr_loader! { SkeletonChildAttrs {
    "TypeFlags" => TypeFlags(String),
    "Properties70" => Properties70,
}}
