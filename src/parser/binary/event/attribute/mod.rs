//! Node attributes.

use parser::binary::Warnings;
use parser::binary::error::{Result, Error, Warning};
use parser::binary::event::NodeHeader;
use parser::binary::reader::{ParserSource, ReadLittleEndian};
use parser::binary::utils::{AttributeValues, AttributeValue};
use self::array::read_array_attribute;
pub use self::array::{ArrayAttribute, ArrayAttributeReader};
use self::special::read_special_attribute;
pub use self::special::{SpecialAttribute, SpecialAttributeType};

mod array;
mod special;


/// Node attribute.
#[derive(Debug)]
pub struct Attributes<'a, R: 'a> {
    /// Number of all attributes.
    num_attributes: u64,
    /// Number of rest attributes.
    rest_attributes: u64,
    /// End offset of the previous attribute.
    prev_attr_end: Option<u64>,
    /// Parser source.
    source: &'a mut R,
    /// Parser warnings.
    warnings: &'a mut Warnings,
}

impl<'a, R: 'a + ParserSource> Attributes<'a, R> {
    /// Returns number of all attributes.
    pub fn num_attributes(&self) -> u64 {
        self.num_attributes
    }

    /// Returns number of rest attributes.
    pub fn rest_attributes(&self) -> u64 {
        self.rest_attributes
    }

    /// Returns the next attribute if available.
    pub fn next_attribute(&mut self) -> Result<Option<Attribute<R>>> {
        if self.rest_attributes == 0 {
            return Ok(None);
        }

        // Skip unread part of the previous attribute if available.
        if let Some(prev_attr_end) = self.prev_attr_end {
            self.source.skip_to(prev_attr_end)?;
            self.prev_attr_end = None;
        }

        self.rest_attributes -= 1;
        let type_code = self.source.read_u8()?;
        let position = self.source.position();
        match type_code {
            // Primitive type attributes.
            b'C' => {
                let raw = self.source.read_u8()?;
                let val = (raw & 0x01) == 1;
                if raw != b'T' && raw != b'Y' {
                    self.warnings.warn(Warning::InvalidBooleanAttributeValue {
                        got: raw,
                        assumed: val,
                        position: position,
                    });
                }
                Ok(Some(PrimitiveAttribute::Bool(val).into()))
            },
            b'Y' => Ok(Some(PrimitiveAttribute::I16(self.source.read_i16()?).into())),
            b'I' => Ok(Some(PrimitiveAttribute::I32(self.source.read_i32()?).into())),
            b'L' => Ok(Some(PrimitiveAttribute::I64(self.source.read_i64()?).into())),
            b'F' => Ok(Some(PrimitiveAttribute::F32(self.source.read_f32()?).into())),
            b'D' => Ok(Some(PrimitiveAttribute::F64(self.source.read_f64()?).into())),
            // Special type attributes.
            b'R' | b'S' => {
                let (attr, end_offset) = read_special_attribute(self.source, type_code)?;
                self.prev_attr_end = Some(end_offset);
                Ok(Some(attr.into()))
            },
            // Array type attributes.
            b'b' | b'i' | b'l' | b'f' | b'd' => {
                let (attr, end_offset) =
                    read_array_attribute(self.source, self.warnings, type_code)?;
                self.prev_attr_end = Some(end_offset);
                Ok(Some(attr.into()))
            },
            // Unknown type attributes.
            _ => {
                Err(Error::InvalidNodeAttributeTypeCode {
                    got: type_code,
                    position: position,
                })
            },
        }
    }

    /// Converts some attributes into values of specific types.
    pub fn convert_into<A: AttributeValues>(&mut self) -> Result<Option<A>> {
        A::from_attributes(self)
    }
}

impl<'a, R: 'a + ParserSource> From<PrimitiveAttribute> for Attribute<'a, R> {
    fn from(a: PrimitiveAttribute) -> Self {
        Attribute::Primitive(a)
    }
}

impl<'a, R: 'a + ParserSource> From<SpecialAttribute<'a, R>> for Attribute<'a, R> {
    fn from(a: SpecialAttribute<'a, R>) -> Self {
        Attribute::Special(a)
    }
}

impl<'a, R: 'a + ParserSource> From<ArrayAttribute<'a, R>> for Attribute<'a, R> {
    fn from(a: ArrayAttribute<'a, R>) -> Self {
        Attribute::Array(a)
    }
}


/// Creates a new `Attributes`.
pub fn new_attributes<'a, R: 'a>(
    source: &'a mut R,
    warnings: &'a mut Warnings,
    header: &NodeHeader
) -> Attributes<'a, R> {
    Attributes {
        num_attributes: header.num_attributes,
        rest_attributes: header.num_attributes,
        prev_attr_end: None,
        source: source,
        warnings: warnings,
    }
}


/// Node attribute.
#[derive(Debug)]
pub enum Attribute<'a, R: 'a> {
    /// Primitive type value.
    Primitive(PrimitiveAttribute),
    /// Array type value.
    Array(ArrayAttribute<'a, R>),
    /// Special type value.
    Special(SpecialAttribute<'a, R>),
}

impl<'a, R: 'a + ParserSource> Attribute<'a, R> {
    /// Converts the attribute into a value of a specific type.
    pub fn convert_into<A: AttributeValue>(self) -> Result<Option<A>> {
        A::from_attribute(self)
    }
}


/// Attribute type of primitive value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveAttribute {
    /// Single `bool`.
    Bool(bool),
    /// Single `i16`.
    I16(i16),
    /// Single `i32`.
    I32(i32),
    /// Single `i64`.
    I64(i64),
    /// Single `f32`.
    F32(f32),
    /// Single `f64`.
    F64(f64),
}

impl PrimitiveAttribute {
    /// Converts floating-point number into `f32`.
    pub fn as_f32(&self) -> Option<f32> {
        match *self {
            PrimitiveAttribute::F32(v) => Some(v),
            PrimitiveAttribute::F64(v) => Some(v as f32),
            _ => None,
        }
    }

    /// Converts floating-point number into `f32`.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            PrimitiveAttribute::F32(v) => Some(v as f64),
            PrimitiveAttribute::F64(v) => Some(v),
            _ => None,
        }
    }
}
