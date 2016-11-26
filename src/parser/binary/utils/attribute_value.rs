//! Traits for types which can be converted from node attributes.

use std::io::Result as IoResult;

use parser::binary::{Result, ParserSource, Attribute};
use parser::binary::{PrimitiveAttribute, ArrayAttribute, SpecialAttributeType};


/// Types which can be converted from a node attribute.
pub trait AttributeValue: Sized {
    /// Reads the given attribute as `Self` type.
    ///
    /// The value type will be strictly checked.
    ///
    /// Returns `Ok(Some(Self))` if successfully converted,
    /// `Ok(None)` if successfully read but the types didn't matched,
    /// `Err(_)` if parse error happened.
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>>;

    /// Reads the given attribute and converts into `Self` type.
    ///
    /// The value type will be loosely checked.
    ///
    /// Returns `Ok(Some(Self))` if successfully converted,
    /// `Ok(None)` if successfully read but the types were incompatible,
    /// `Err(_)` if parse error happened.
    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>>;
}

// Simply ignore the attribute.
impl AttributeValue for () {
    fn from_attribute<R: ParserSource>(_: Attribute<R>) -> Result<Option<Self>> {
        Ok(Some(()))
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for bool {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::Bool(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for i16 {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::I16(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for i32 {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::I32(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        match attr {
            Attribute::Primitive(PrimitiveAttribute::I16(val)) => Ok(Some(val as i32)),
            Attribute::Primitive(PrimitiveAttribute::I32(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for i64 {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::I64(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        match attr {
            Attribute::Primitive(PrimitiveAttribute::I16(val)) => Ok(Some(val as i64)),
            Attribute::Primitive(PrimitiveAttribute::I32(val)) => Ok(Some(val as i64)),
            Attribute::Primitive(PrimitiveAttribute::I64(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for f32 {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::F32(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(attr) = attr {
            Ok(attr.as_f32())
        } else {
            Ok(None)
        }
    }
}

impl AttributeValue for f64 {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(PrimitiveAttribute::F64(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Primitive(attr) = attr {
            Ok(attr.as_f64())
        } else {
            Ok(None)
        }
    }
}

impl AttributeValue for Vec<i32> {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Array(ArrayAttribute::I32(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for Vec<i64> {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Array(ArrayAttribute::I64(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        match attr {
            Attribute::Array(ArrayAttribute::I32(arr)) => {
                Ok(Some(arr.into_iter().map(|v| v.map(Into::into)).collect::<IoResult<_>>()?))
            },
            Attribute::Array(ArrayAttribute::I64(arr)) => Ok(Some(arr.into_vec()?)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for Vec<f32> {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Array(ArrayAttribute::F32(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        match attr {
            Attribute::Array(ArrayAttribute::F32(arr)) => Ok(Some(arr.into_vec()?)),
            Attribute::Array(ArrayAttribute::F64(arr)) => {
                Ok(Some(arr.into_iter().map(|v| v.map(|v| v as f32)).collect::<IoResult<_>>()?))
            },
            _ => Ok(None),
        }
    }
}

impl AttributeValue for Vec<f64> {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Array(ArrayAttribute::F64(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        match attr {
            Attribute::Array(ArrayAttribute::F32(arr)) => {
                Ok(Some(arr.into_iter().map(|v| v.map(Into::into)).collect::<IoResult<_>>()?))
            },
            Attribute::Array(ArrayAttribute::F64(arr)) => Ok(Some(arr.into_vec()?)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for String {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::String {
                return Ok(Some(val.into_string()?));
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for Vec<u8> {
    fn from_attribute<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::Binary {
                return Ok(Some(val.into_vec()?));
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R: ParserSource>(attr: Attribute<R>) -> Result<Option<Self>> {
        if let Attribute::Special(val) = attr {
            Ok(Some(val.into_vec()?))
        } else {
            Ok(None)
        }
    }
}
