//! Deformer-related enums of FBX 7.4 or later.

use std::str::FromStr;

use parser::binary::Result as ParserResult;
use parser::binary::{ParserSource, Attribute, SpecialAttributeType};
use parser::binary::utils::AttributeValue;
use loader::enums::NoSuchVariant;


/// Skinning type.
///
/// See [FBX 2018 Developer Help: `FbxSkin` Class
/// Reference](https://help.autodesk.com/cloudhelp/2018/ENU/FBX-Developer-Help/cpp_ref/class_fbx_skin.html#ad5d0e87f61ba99c47a539492df7917a1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkinningType {
    /// Rigid skinning.
    Rigid,
    /// Linear skinning.
    Linear,
    /// Dual quaternion skinning.
    DualQuaternion,
    /// Blend linear and dual quaternion skinning according to blend weights.
    Blend,
}

impl FromStr for SkinningType {
    type Err = NoSuchVariant;

    /// Get a mapping mode value from the property value of a `MappingInformationType` node.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rigid" => Ok(SkinningType::Rigid),
            "Linear" => Ok(SkinningType::Linear),
            "DualQuaternion" => Ok(SkinningType::DualQuaternion),
            "Blend" => Ok(SkinningType::Blend),
            _ => Err(NoSuchVariant),
        }
    }
}

impl AttributeValue for SkinningType {
    fn from_attribute<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::String {
                return Ok(Self::from_str(&val.into_string()?).ok());
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Self::from_attribute(attr)
    }
}
