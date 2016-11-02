//! Node attributes.

use std::io::Read;

use parser::binary::BinaryParser;
use parser::binary::error::{Result, Warning};
use parser::binary::event::NodeHeader;
pub use self::array::ArrayAttribute;
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
    /// Parser.
    parser: &'a mut BinaryParser<R>,
}

impl<'a, R: 'a + Read> Attributes<'a, R> {
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

        self.rest_attributes -= 1;
        let type_code = try!(self.parser.source.read_u8());
        let position = self.parser.source.count();
        match type_code {
            b'C' => {
                let raw = try!(self.parser.source.read_u8());
                let val = (raw & 0x01) == 1;
                if raw != b'T' && raw != b'Y' {
                    self.parser.warn(Warning::InvalidBooleanAttributeValue {
                        got: raw,
                        assumed: val,
                        position: position,
                    });
                }
                Ok(Some(Attribute::Primitive(PrimitiveAttribute::Bool(val))))
            },
            b'Y' => Ok(Some(Attribute::Primitive(PrimitiveAttribute::I16(try!(self.parser.source.read_i16()))))),
            b'I' => Ok(Some(Attribute::Primitive(PrimitiveAttribute::I32(try!(self.parser.source.read_i32()))))),
            b'L' => Ok(Some(Attribute::Primitive(PrimitiveAttribute::I64(try!(self.parser.source.read_i64()))))),
            b'F' => Ok(Some(Attribute::Primitive(PrimitiveAttribute::F32(try!(self.parser.source.read_f32()))))),
            b'D' => Ok(Some(Attribute::Primitive(PrimitiveAttribute::F64(try!(self.parser.source.read_f64()))))),
            _ => unimplemented!(),
        }
    }
}


/// Creates a new `Attributes`.
pub fn new_attributes<'a, R: 'a>(
    parser: &'a mut BinaryParser<R>,
    header: &NodeHeader
) -> Attributes<'a, R> {
    Attributes {
        num_attributes: header.num_attributes,
        rest_attributes: header.num_attributes,
        parser: parser,
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
