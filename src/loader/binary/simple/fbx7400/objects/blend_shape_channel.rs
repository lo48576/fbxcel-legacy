//! `BlendShapeChannel` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Deformer` node with class=`SubDeformer`, subclass=`BlendShapeChannel`.
///
/// `FbxBlendShapeChannel` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct BlendShapeChannel {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `DeformPercent`.
    ///
    /// This property also exists in `Properties70`... which should be used?
    pub deform_percent: f64,
    /// `FullWeights`.
    pub full_weights: Vec<f64>,
    /// `Properties70`.
    pub properties: Properties70,
}

impl BlendShapeChannel {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut deform_percent = None;
        let mut full_weights = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, BlendShapeChannelChildAttrs::load);
            match node_type {
                BlendShapeChannelChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                BlendShapeChannelChildAttrs::DeformPercent(v) => {
                    deform_percent = Some(v);
                    parser.skip_current_node()?;
                },
                BlendShapeChannelChildAttrs::FullWeights(v) => {
                    full_weights = Some(v);
                    parser.skip_current_node()?;
                },
                BlendShapeChannelChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(BlendShapeChannel {
            id: obj_props.id,
            version: ensure_node_exists!(version,
                                         node_msg!(Deformer, SubDeformer, BlendShapeChannel)),
            deform_percent: ensure_node_exists!(deform_percent,
                                                node_msg!(Deformer,
                                                          SubDeformer,
                                                          BlendShapeChannel)),
            full_weights: ensure_node_exists!(full_weights,
                                              node_msg!(Deformer, SubDeformer, BlendShapeChannel)),
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Deformer")
            .and_then(|ot| ot.property_template.get("FbxBlendShapeChannel"))
    }

    /// `DeformPercent`.
    pub fn get_deform_percent(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        // Which should be used, `properties70` or `self.deform_percent`?
        get_property!(self.properties, defaults, values_f64, "DeformPercent").map(|v| *v.value())
    }
}


child_attr_loader! { BlendShapeChannelChildAttrs {
    "Version" => Version(i32),
    "DeformPercent" => DeformPercent(f64),
    "FullWeights" => FullWeights(Vec<f64>),
    "Properties70" => Properties70,
}}
