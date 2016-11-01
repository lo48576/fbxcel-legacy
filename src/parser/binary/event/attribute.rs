//! Node attributes.

use std::io::Read;

use parser::binary::BinaryParser;
use parser::binary::error::Result;
use parser::binary::event::NodeHeader;


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
    /// Dummy.
    _Dummy(&'a mut R),
}
