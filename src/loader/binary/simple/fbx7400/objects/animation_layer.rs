//! `AnimationLayer` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `AnimationLayer` node with class=`AnimLayer`, subclass=``.
///
/// `FbxAnimLayer` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationLayer {
    /// Object id.
    pub id: i64,
    /// `Properties70`.
    pub properties: Properties70,
}

impl AnimationLayer {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, AnimationLayerChildAttrs::load);
            match node_type {
                AnimationLayerChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(AnimationLayer {
            id: obj_props.id,
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("AnimationLayer")
            .and_then(|ot| ot.property_template.get("FbxAnimLayer"))
    }

    /// `Weight`.
    pub fn get_weight(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "Weight").map(|v| *v.value())
    }

    /// `Mute`.
    pub fn get_mute(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "Mute").map(|v| *v.value() != 0)
    }

    /// `Solo`.
    pub fn get_solo(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "Solo").map(|v| *v.value() != 0)
    }

    /// `Lock`.
    pub fn get_lock(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "Lock").map(|v| *v.value() != 0)
    }

    /// `Color`.
    pub fn get_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "Color").map(|v| *v.value())
    }

    /// `BlendMode`.
    pub fn get_blend_mode(&self, defaults: &Option<&Properties70>) -> Option<BlendMode> {
        get_property!(self.properties, defaults, values_i64, "BlendMode")
            .and_then(|v| BlendMode::from_i64(*v.value()))
    }

    /// `RotationAccumulationMode`.
    pub fn get_rotation_accumulation_mode(
        &self,
        defaults: &Option<&Properties70>
    ) -> Option<RotationAccumulationMode> {
        get_property!(self.properties,
                      defaults,
                      values_i64,
                      "RotationAccumulationMode")
            .and_then(|v| RotationAccumulationMode::from_i64(*v.value()))
    }

    /// `ScaleAccumulationMode`.
    pub fn get_scale_accumulation_mode(
        &self,
        defaults: &Option<&Properties70>
    ) -> Option<ScaleAccumulationMode> {
        get_property!(self.properties,
                      defaults,
                      values_i64,
                      "ScaleAccumulationMode")
            .and_then(|v| ScaleAccumulationMode::from_i64(*v.value()))
    }

    /// `BlendModeBypass`.
    pub fn get_blend_mode_bypass(&self, defaults: &Option<&Properties70>) -> Option<u64> {
        get_property!(self.properties, defaults, values_i64, "BlendModeBypass")
            .map(|v| *v.value() as u64)
    }
}


child_attr_loader! { AnimationLayerChildAttrs {
    "Properties70" => Properties70,
}}


/// `FbxAnimLayer::EBlendMode` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// `eBlendAdditive`.
    Additive = 0,
    /// `eBlendOverride`.
    Override = 1,
    /// `eBlendOverridePassthrough`.
    OverridePassthrough = 2,
}

impl BlendMode {
    /// Creates an enum value from the given value.
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            v if v == BlendMode::Additive as i64 => Some(BlendMode::Additive),
            v if v == BlendMode::Override as i64 => Some(BlendMode::Override),
            v if v == BlendMode::OverridePassthrough as i64 => Some(BlendMode::OverridePassthrough),
            _ => None,
        }
    }
}


/// `FbxAnimLayer::ERotationAccumulationMode` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotationAccumulationMode {
    /// `eRotationByLayer`.
    ByLayer = 0,
    /// `eRotationByChannel`.
    ByChannel = 1,
}

impl RotationAccumulationMode {
    /// Creates an enum value from the given value.
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            v if v == RotationAccumulationMode::ByLayer as i64 => {
                Some(RotationAccumulationMode::ByLayer)
            },
            v if v == RotationAccumulationMode::ByChannel as i64 => {
                Some(RotationAccumulationMode::ByChannel)
            },
            _ => None,
        }
    }
}


/// `FbxAnimLayer::EScaleAccumulationMode` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleAccumulationMode {
    /// `eScaleMultiply`.
    Multiply = 0,
    /// `eScaleAdditive`.
    Additive = 1,
}

impl ScaleAccumulationMode {
    /// Creates an enum value from the given value.
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            v if v == ScaleAccumulationMode::Multiply as i64 => {
                Some(ScaleAccumulationMode::Multiply)
            },
            v if v == ScaleAccumulationMode::Additive as i64 => {
                Some(ScaleAccumulationMode::Additive)
            },
            _ => None,
        }
    }
}
