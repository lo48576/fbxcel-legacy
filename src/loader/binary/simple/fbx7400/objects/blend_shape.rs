//! `BlendShape` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Deformer` node with class=`Deformer`, subclass=`BlendShape`.
///
/// `FbxBlendShape` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct BlendShape {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `Properties70`.
    pub properties: Properties70,
}

impl BlendShape {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, BlendShapeChildAttrs::load);
            match node_type {
                BlendShapeChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                BlendShapeChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(BlendShape {
            id: obj_props.id,
            version: ensure_node_exists!(version, node_msg!(Deformer, Deformer, Skin)),
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Deformer")
            .and_then(|ot| ot.property_template.get("FbxBlendShape"))
    }
}


child_attr_loader! { BlendShapeChildAttrs {
    "Version" => Version(i32),
    "Properties70" => Properties70,
}}
