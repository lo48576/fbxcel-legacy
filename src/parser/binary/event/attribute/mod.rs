//! Node attributes.

use std::io::Read;

use parser::binary::BinaryParser;
use parser::binary::error::Result;
use parser::binary::event::NodeHeader;
pub use self::array::ArrayAttribute;
pub use self::special::{SpecialAttribute, SpecialAttributeType};

mod array;
mod special;


/// Node attribute.
#[derive(Debug)]
pub struct Attributes<'a, R: 'a + Read> {
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
        unimplemented!()
    }
}


/// Creates a new `Attributes`.
pub fn new_attributes<'a, R: 'a + Read>(
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
pub enum Attribute<'a, R: 'a + Read> {
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
