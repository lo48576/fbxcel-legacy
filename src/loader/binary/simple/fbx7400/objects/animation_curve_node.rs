//! `AnimationCurveNode` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `AnimationCurveNode` node with class=`AnimCurveNode`, subclass=``.
///
/// `FbxAnimCurveNode` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationCurveNode {
    /// Object id.
    pub id: i64,
    /// `Properties70`.
    pub properties: Properties70,
}

impl AnimationCurveNode {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, AnimationCurveNodeChildAttrs::load);
            match node_type {
                AnimationCurveNodeChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(AnimationCurveNode {
            id: obj_props.id,
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("AnimationCurveNode")
            .and_then(|ot| ot.property_template.get("FbxAnimCurveNode"))
    }

    /// `d|X`.
    pub fn get_d_x(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "d|X").map(|v| *v.value())
    }

    /// `d|Y`.
    pub fn get_d_y(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "d|Y").map(|v| *v.value())
    }

    /// `d|Z`.
    pub fn get_d_z(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "d|Z").map(|v| *v.value())
    }
}


child_attr_loader! { AnimationCurveNodeChildAttrs {
    "Properties70" => Properties70,
}}
