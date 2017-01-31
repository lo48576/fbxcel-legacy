//! `DisplayLayer` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `CollectionExclusive` node with class=`DisplayLayer`, subclass=`DisplayLayer`.
///
/// `FbxDisplayLayer` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayLayer {
    /// Object ID.
    pub id: i64,
    /// Properties.
    pub properties: Properties70,
}

impl DisplayLayer {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, DisplayLayerChildAttrs::load);
            match node_type {
                DisplayLayerChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(DisplayLayer {
            id: obj_props.id,
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("CollectionExclusive")
            .and_then(|ot| ot.property_template.get("FbxDisplayLayer"))
    }

    /// `Color`.
    pub fn get_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "Color").map(|v| *v.value())
    }

    /// `Show`.
    pub fn get_show(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "Show").map(|v| *v.value() != 0)
    }

    /// `Freeze`.
    pub fn get_freeze(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "Freeze").map(|v| *v.value() != 0)
    }

    /// `LODBox`.
    pub fn get_lod_box(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "LODBox").map(|v| *v.value() != 0)
    }
}


child_attr_loader! { DisplayLayerChildAttrs {
    "Properties70" => Properties70,
}}
