//! Traits for types which can be converted from node attributes.

use std::io::Result as IoResult;

use parser::binary::{Result, ParserSource, Attributes, Attribute};
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
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>> where R: ParserSource;

    /// Reads the given attribute and converts into `Self` type.
    ///
    /// The value type will be loosely checked.
    ///
    /// Returns `Ok(Some(Self))` if successfully converted,
    /// `Ok(None)` if successfully read but the types were incompatible,
    /// `Err(_)` if parse error happened.
    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>> where R: ParserSource;
}

// Simply ignore the attribute.
impl AttributeValue for () {
    fn from_attribute<R>(_: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Ok(Some(()))
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for bool {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::Bool(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for i16 {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::I16(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for i32 {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::I32(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        match attr {
            Attribute::Primitive(PrimitiveAttribute::I16(val)) => Ok(Some(val as i32)),
            Attribute::Primitive(PrimitiveAttribute::I32(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for i64 {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::I64(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        match attr {
            Attribute::Primitive(PrimitiveAttribute::I16(val)) => Ok(Some(val as i64)),
            Attribute::Primitive(PrimitiveAttribute::I32(val)) => Ok(Some(val as i64)),
            Attribute::Primitive(PrimitiveAttribute::I64(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for f32 {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::F32(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(attr) = attr {
            Ok(attr.as_f32())
        } else {
            Ok(None)
        }
    }
}

impl AttributeValue for f64 {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(PrimitiveAttribute::F64(val)) = attr {
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Primitive(attr) = attr {
            Ok(attr.as_f64())
        } else {
            Ok(None)
        }
    }
}

impl AttributeValue for Vec<i32> {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Array(ArrayAttribute::I32(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for Vec<i64> {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Array(ArrayAttribute::I64(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        match attr {
            Attribute::Array(ArrayAttribute::I32(arr)) => {
                Ok(Some(arr.into_iter()
                    .map(|v| v.map(Into::into))
                    .collect::<IoResult<_>>()?))
            },
            Attribute::Array(ArrayAttribute::I64(arr)) => Ok(Some(arr.into_vec()?)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for Vec<f32> {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Array(ArrayAttribute::F32(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        match attr {
            Attribute::Array(ArrayAttribute::F32(arr)) => Ok(Some(arr.into_vec()?)),
            Attribute::Array(ArrayAttribute::F64(arr)) => {
                Ok(Some(arr.into_iter()
                    .map(|v| v.map(|v| v as f32))
                    .collect::<IoResult<_>>()?))
            },
            _ => Ok(None),
        }
    }
}

impl AttributeValue for Vec<f64> {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Array(ArrayAttribute::F64(arr)) = attr {
            Ok(Some(arr.into_vec()?))
        } else {
            Ok(None)
        }
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        match attr {
            Attribute::Array(ArrayAttribute::F32(arr)) => {
                Ok(Some(arr.into_iter()
                    .map(|v| v.map(Into::into))
                    .collect::<IoResult<_>>()?))
            },
            Attribute::Array(ArrayAttribute::F64(arr)) => Ok(Some(arr.into_vec()?)),
            _ => Ok(None),
        }
    }
}

impl AttributeValue for String {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::String {
                return Ok(Some(val.into_string()?));
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        Self::from_attribute(attr)
    }
}

impl AttributeValue for Vec<u8> {
    fn from_attribute<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Special(val) = attr {
            if val.value_type() == SpecialAttributeType::Binary {
                return Ok(Some(val.into_vec()?));
            }
        }
        Ok(None)
    }

    fn from_attribute_loose<R>(attr: Attribute<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        if let Attribute::Special(val) = attr {
            Ok(Some(val.into_vec()?))
        } else {
            Ok(None)
        }
    }
}


/// Attribute values list (tuple).
pub trait AttributeValues: Sized {
    /// Construct `Self` via a conversion if possible.
    ///
    /// Returns `Ok(Some(values))` if successfully converted,
    /// `Ok(None)` if the parsing is successfully done but types are incompatible,
    /// `Err(_)` if the parsing failed.
    fn from_attributes<R>(attrs: &mut Attributes<R>) -> Result<Option<Self>> where R: ParserSource;
}

macro_rules! impl_attribute_values {
    ($($name:ident: $t:ident),+,) => {
        impl<$($t: AttributeValue),+> AttributeValues for ($($t),+,) {
            fn from_attributes<R>(attrs: &mut Attributes<R>) -> Result<Option<Self>>
                where R: ParserSource
            {
                $(
                    let $name = {
                        let attr = match attrs.next_attribute()? {
                            Some(attr) => attr,
                            None => return Ok(None),
                        };
                        match $t::from_attribute(attr)? {
                            Some(val) => val,
                            None => return Ok(None),
                        }
                    };
                )+
                Ok(Some(($($name),+,)))
            }
        }
    }
}

impl<T: AttributeValue> AttributeValues for T {
    fn from_attributes<R>(attrs: &mut Attributes<R>) -> Result<Option<Self>>
        where R: ParserSource
    {
        <(T,)>::from_attributes(attrs).map(|v_opt| v_opt.map(|v| v.0))
    }
}

impl_attribute_values! {
    t1: T1,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
    t6: T6,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
    t6: T6,
    t7: T7,
}
impl_attribute_values! {
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
    t6: T6,
    t7: T7,
    t8: T8,
}
