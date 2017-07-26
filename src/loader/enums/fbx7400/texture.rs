//! Texture-related enums of FBX 7.4 or later.

use parser::binary::Result as ParserResult;
use parser::binary::{ParserSource, Attribute};
use parser::binary::utils::AttributeValue;


/// Blend mode of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    /// Use alpha.
    Translucent,
    /// Add colors.
    Additive,
    /// Multiply colors.
    Modulate,
    /// Multiply the texture colors by two and then multiply colors.
    Modulate2,
    /// Opaque texture.
    Over,
}

impl BlendMode {
    /// Converts the given `i32` value into `BlendMode`.
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(BlendMode::Translucent),
            1 => Some(BlendMode::Additive),
            2 => Some(BlendMode::Modulate),
            3 => Some(BlendMode::Modulate2),
            4 => Some(BlendMode::Over),
            _ => None,
        }
    }
}

impl AttributeValue for BlendMode {
    fn from_attribute<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Ok(i32::from_attribute(attr)?.and_then(Self::from_i32))
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Ok(i32::from_attribute_loose(attr)?.and_then(Self::from_i32))
    }
}


/// Wrap mode of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WrapMode {
    /// Repeat the texture.
    ///
    /// This is the default value.
    Repeat,
    /// Clamp.
    Clamp,
}

impl WrapMode {
    /// Converts the given `i32` value into `WrapMode`.
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(WrapMode::Repeat),
            1 => Some(WrapMode::Clamp),
            _ => None,
        }
    }
}

impl Default for WrapMode {
    fn default() -> Self {
        WrapMode::Repeat
    }
}

impl AttributeValue for WrapMode {
    fn from_attribute<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Ok(i32::from_attribute(attr)?.and_then(Self::from_i32))
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> ParserResult<Option<Self>>
    where
        R: ParserSource,
    {
        Ok(i32::from_attribute_loose(attr)?.and_then(Self::from_i32))
    }
}
