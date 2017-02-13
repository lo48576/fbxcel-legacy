//! `Material` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Material` node with class=`Material`, subclass=``.
///
/// `FbxSurfaceMaterial` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `MultiLayer`.
    pub multi_layer: bool,
    /// `ShadingModel`.
    pub shading_model: ShadingModel,
    /// `Properties70`.
    pub properties: Properties70,
}

impl Material {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut multi_layer = None;
        let mut shading_model = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, MaterialChildAttrs::load);
            match node_type {
                MaterialChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                MaterialChildAttrs::MultiLayer(v) => {
                    multi_layer = Some(v != 0);
                    parser.skip_current_node()?;
                },
                MaterialChildAttrs::ShadingModel(v) => {
                    use std::str::FromStr;

                    let v = ShadingModel::from_str(&v).ok();
                    if v.is_none() {
                        return Err(Error::InvalidAttribute("ShadingModel".to_owned()));
                    }
                    shading_model = v;
                    parser.skip_current_node()?;
                },
                MaterialChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Material {
            id: obj_props.id,
            version: ensure_node_exists! { version,
                "Material (class=`Material`, subclass=``)".to_owned()},
            multi_layer: ensure_node_exists! { multi_layer,
                "Material (class=`Material`, subclass=``)".to_owned()},
            shading_model: ensure_node_exists! { shading_model,
                "Material (class=`Material`, subclass=``)".to_owned()},
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property<'a>(
        definitions: &'a Definitions,
        class_name: Option<&str>
    ) -> Option<&'a Properties70> {
        definitions.object_types
            .get("Material")
            .and_then(|ot| class_name.and_then(|cn| ot.property_template.get(cn)))
    }

    /// `ShadingModel`.
    pub fn get_shading_model(&self, defaults: &Option<&Properties70>) -> Option<String> {
        get_property!(self.properties, defaults, values_string, "ShadingModel")
            .map(|v| v.value().clone())
    }

    /// `MultiLayer`.
    pub fn get_bool(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "MultiLayer").map(|v| *v.value() != 0)
    }

    /// `EmissiveColor`.
    pub fn get_emissive_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "EmissiveColor").map(|v| *v.value())
    }

    /// `EmissiveFactor`.
    pub fn get_emissive_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "EmissiveFactor").map(|v| *v.value())
    }

    /// `AmbientColor`.
    pub fn get_ambient_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "AmbientColor").map(|v| *v.value())
    }

    /// `AmbientFactor`.
    pub fn get_ambient_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "AmbientFactor").map(|v| *v.value())
    }

    /// `DiffuseColor`.
    pub fn get_diffuse_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "DiffuseColor").map(|v| *v.value())
    }

    /// `DiffuseFactor`.
    pub fn get_diffuse_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "DiffuseFactor").map(|v| *v.value())
    }

    /// `SpecularColor`.
    pub fn get_specular_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "SpecularColor").map(|v| *v.value())
    }

    /// `SpecularFactor`.
    pub fn get_specular_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "SpecularFactor").map(|v| *v.value())
    }

    /// `Shininess`.
    pub fn get_shininess(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "Shininess").map(|v| *v.value())
    }

    /// `Bump`.
    pub fn get_bump(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "Bump").map(|v| *v.value())
    }

    /// `BumpFactor`.
    pub fn get_bump_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "BumpFactor").map(|v| *v.value())
    }

    /// `NormalMap`.
    pub fn get_normal_map(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "NormalMap").map(|v| *v.value())
    }

    /// `TransparentColor`.
    pub fn get_transparent_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "TransparentColor")
            .map(|v| *v.value())
    }

    /// `TransparencyFactor`.
    pub fn get_transparency_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "TransparencyFactor")
            .map(|v| *v.value())
    }

    /// `Reflection`.
    pub fn get_reflection(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "Reflection").map(|v| *v.value())
    }

    /// `ReflectionFactor`.
    pub fn get_reflection_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "ReflectionFactor").map(|v| *v.value())
    }

    /// `DisplacementColor`.
    pub fn get_displacement_color(&self, defaults: &Option<&Properties70>) -> Option<[f64; 3]> {
        get_property!(self.properties, defaults, values_f64_3, "DisplacementColor")
            .map(|v| *v.value())
    }

    /// `DisplacementFactor`.
    pub fn get_displacement_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties, defaults, values_f64, "DisplacementFactor")
            .map(|v| *v.value())
    }

    /// `VectorDisplacementColor`.
    pub fn get_vector_displacement_color(
        &self,
        defaults: &Option<&Properties70>
    ) -> Option<[f64; 3]> {
        get_property!(self.properties,
                      defaults,
                      values_f64_3,
                      "VectorDisplacementColor")
            .map(|v| *v.value())
    }

    /// `VectorDisplacementFactor`.
    pub fn get_vector_displacement_factor(&self, defaults: &Option<&Properties70>) -> Option<f64> {
        get_property!(self.properties,
                      defaults,
                      values_f64,
                      "VectorDisplacementFactor")
            .map(|v| *v.value())
    }
}


child_attr_loader! { MaterialChildAttrs {
    "Version" => Version(i32),
    "MultiLayer" => MultiLayer(i32),
    "ShadingModel" => ShadingModel(String),
    "Properties70" => Properties70,
}}


/// A value of `ShadingModel`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShadingModel {
    /// `lambert`.
    Lambert,
    /// `phong`.
    Phong,
    /// `unknown`.
    Unknown,
}

impl ShadingModel {
    /// Returns class name of FBX.
    pub fn class_name(&self) -> Option<&str> {
        match *self {
            ShadingModel::Lambert => Some("FbxSurfaceLambert"),
            ShadingModel::Phong => Some("FbxSurfacePhong"),
            ShadingModel::Unknown => None,
        }
    }
}

impl ::std::str::FromStr for ShadingModel {
    type Err = ();

    /// Creates a new `ShadingModel` from the string (used in FBX).
    fn from_str(v: &str) -> ::std::result::Result<Self, Self::Err> {
        match v {
            "lambert" | "Lambert" => Ok(ShadingModel::Lambert),
            "phong" | "Phong" => Ok(ShadingModel::Phong),
            "unknown" | "Unknown" => Ok(ShadingModel::Unknown),
            _ => Err(()),
        }
    }
}
