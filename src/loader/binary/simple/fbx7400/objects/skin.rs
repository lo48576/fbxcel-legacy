//! `Skin` object.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::{Properties70, Definitions};
use loader::binary::simple::fbx7400::objects::ObjectProperties;


/// `Deformer` node with class=`Deformer`, subclass=`Skin`.
///
/// `FbxSkin` of FBX SDK (2017).
#[derive(Debug, Clone, PartialEq)]
pub struct Skin {
    /// Object id.
    pub id: i64,
    /// `Version`.
    pub version: i32,
    /// `Link_DeformAcuracy`.
    ///
    /// Note that "Acuracy" (FBX really uses this!)
    pub link_deform_accuracy: f64,
    /// `SkinningType`.
    pub skinning_type: Option<SkinningType>,
    /// `Properties70`.
    pub properties: Properties70,
}

impl Skin {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        obj_props: &ObjectProperties
    ) -> Result<Self> {
        let mut version = None;
        let mut link_deform_accuracy = None;
        let mut skinning_type = None;
        let mut properties = None;

        loop {
            let node_type = try_get_node_attrs!(parser, SkinChildAttrs::load);
            match node_type {
                SkinChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                SkinChildAttrs::LinkDeformAccuracy(v) => {
                    link_deform_accuracy = Some(v);
                    parser.skip_current_node()?;
                },
                SkinChildAttrs::SkinningType(v) => {
                    use std::str::FromStr;

                    let v = SkinningType::from_str(&v).ok();
                    if v.is_none() {
                        return Err(Error::InvalidAttribute("SkinningType".to_owned()));
                    }
                    skinning_type = v;
                    parser.skip_current_node()?;
                },
                SkinChildAttrs::Properties70 => {
                    properties = Some(Properties70::load(parser.subtree_parser())?);
                },
            }
        }
        Ok(Skin {
            id: obj_props.id,
            version: ensure_node_exists!(version, node_msg!(Deformer, Deformer, Skin)),
            link_deform_accuracy: ensure_node_exists!(link_deform_accuracy,
                                                      node_msg!(Deformer, Deformer, Skin)),
            skinning_type: skinning_type,
            properties: properties.unwrap_or_default(),
        })
    }

    /// Returns property template for this type of node.
    pub fn template_property(definitions: &Definitions) -> Option<&Properties70> {
        definitions.object_types
            .get("Model")
            .and_then(|ot| ot.property_template.get("FbxNode"))
    }
}


child_attr_loader! { SkinChildAttrs {
    "Version" => Version(i32),
    "Link_DeformAcuracy" => LinkDeformAccuracy(f64),
    "SkinningType" => SkinningType(String),
    "Properties70" => Properties70,
}}


/// Skinning type.
///
/// `FbxSkin::EType` of FBX SDK (2017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinningType {
    /// `EType::eRigid`.
    Rigid,
    /// `EType::eLinear`.
    Linear,
    /// `EType::eDualQuaternion`.
    DualQuaternion,
    /// `EType::eBlend`.
    Blend,
}

impl ::std::str::FromStr for SkinningType {
    type Err = ();

    /// Creates a new `SkinningType` from the string (used in FBX).
    fn from_str(v: &str) -> ::std::result::Result<Self, Self::Err> {
        match v {
            "Rigid" => Ok(SkinningType::Rigid),
            "Linear" => Ok(SkinningType::Linear),
            "DualQuaternion" => Ok(SkinningType::DualQuaternion),
            "Blend" => Ok(SkinningType::Blend),
            _ => Err(()),
        }
    }
}
