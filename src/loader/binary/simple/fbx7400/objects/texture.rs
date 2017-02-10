//! `Texture` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Texture` node with class=`Texture`, subclass=``.
///
/// `FbxTexture` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// Object id.
    pub id: i64,
    /// `Type`.
    pub type_: String,
    /// `Version`.
    pub version: i32,
    /// `TextureName`.
    pub texture_name: String,
    /// `Media`.
    pub media: String,
    /// `FileName`.
    ///
    /// `N` is capital.
    pub filename: String,
    /// `RelativeFilename`.
    ///
    /// `n` is lower letter.
    pub relative_filename: String,
    /// `ModelUVTranslation`.
    pub model_uv_translation: [f64; 2],
    /// `ModelUVScaling`.
    pub model_uv_scaling: [f64; 2],
    /// `Texture_Alpha_Source`.
    ///
    /// Currently not used (and always `None`)?
    pub texture_alpha_source: String,
    /// `Cropping`.
    pub cropping: [i32; 4],
    /// `Properties70`.
    pub properties: Properties70,
}

impl Texture {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut type_ = None;
        let mut version = None;
        let mut texture_name = None;
        let mut media = None;
        let mut filename = None;
        let mut relative_filename = None;
        let mut model_uv_translation = None;
        let mut model_uv_scaling = None;
        let mut texture_alpha_source = None;
        let mut cropping = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, TextureChildAttrs::load);
            match node_type {
                TextureChildAttrs::Type(v) => {
                    type_ = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::TextureName(v) => {
                    texture_name = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::Media(v) => {
                    media = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::FileName(v) => {
                    filename = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::RelativeFilename(v) => {
                    relative_filename = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::ModelUvTranslation(v) => {
                    model_uv_translation = Some([v.0, v.1]);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::ModelUvScaling(v) => {
                    model_uv_scaling = Some([v.0, v.1]);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::TextureAlphaSource(v) => {
                    texture_alpha_source = Some(v);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::Cropping(v) => {
                    cropping = Some([v.0, v.1, v.2, v.3]);
                    parser.skip_current_node()?;
                },
                TextureChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Texture {
            id: obj_props.id,
            type_: ensure_node_exists! { type_,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            version: ensure_node_exists! { version,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            texture_name: ensure_node_exists! { texture_name,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            media: ensure_node_exists! { media,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            filename: ensure_node_exists! { filename,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            relative_filename: ensure_node_exists! { relative_filename,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            model_uv_translation: ensure_node_exists! { model_uv_translation,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            model_uv_scaling: ensure_node_exists! { model_uv_scaling,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            texture_alpha_source: ensure_node_exists! { texture_alpha_source,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            cropping: ensure_node_exists! { cropping,
                "Texture (class=`Texture`, subclass=``)".to_owned()},
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        // FIXME: Is it ok to use `FbxFileTexture` unconditionally?
        definitions.object_types
            .get("Texture")
            .and_then(|ot| ot.property_template.get("FbxFileTexture"))
    }

    /// `CurrentTextureBlendMode`.
    pub fn get_current_texture_blend_mode(
        &self,
        defaults: &Option<&Properties70>
    ) -> Option<BlendMode> {
        get_property!(self.properties,
                      defaults,
                      values_i64,
                      "CurrentTextureBlendMode")
            .and_then(|v| BlendMode::from_i64(*v.value()))
    }

    /// `UVSet`.
    pub fn get_uv_set(&self, defaults: &Option<&Properties70>) -> Option<String> {
        get_property!(self.properties, defaults, values_string, "UVSet").map(|v| v.value().clone())
    }

    /// `UseMaterial`.
    pub fn get_use_material(&self, defaults: &Option<&Properties70>) -> Option<bool> {
        get_property!(self.properties, defaults, values_i64, "UseMaterial").map(|v| *v.value() != 0)
    }
}


child_attr_loader! { TextureChildAttrs {
    "Type" => Type(String),
    "Version" => Version(i32),
    "TextureName" => TextureName(String),
    "Media" => Media(String),
    "FileName" => FileName(String),
    "RelativeFilename" => RelativeFilename(String),
    "ModelUVTranslation" => ModelUvTranslation((f64, f64)),
    "ModelUVScaling" => ModelUvScaling((f64, f64)),
    "Texture_Alpha_Source" => TextureAlphaSource(String),
    "Cropping" => Cropping((i32, i32, i32, i32)),
    "Properties70" => Properties70,
}}


/// `FbxTexture::EBlendMode` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// `eTranslucent`.
    Translucent = 0,
    /// `eAdditive`.
    Additive = 1,
    /// `eModulate`.
    Modulate = 2,
    /// `eModulate2`.
    Modulate2 = 3,
    /// `eOver`.
    Over = 4,
}

impl BlendMode {
    /// Creates an enum value from the given value.
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            v if v == BlendMode::Translucent as i64 => Some(BlendMode::Translucent),
            v if v == BlendMode::Additive as i64 => Some(BlendMode::Additive),
            v if v == BlendMode::Modulate as i64 => Some(BlendMode::Modulate),
            v if v == BlendMode::Modulate2 as i64 => Some(BlendMode::Modulate2),
            v if v == BlendMode::Over as i64 => Some(BlendMode::Over),
            _ => None,
        }
    }
}
