//! Generic node and node attribute.

use parser::binary::{Parser, ParserSource, Event, Attributes, Attribute, FbxFooter};
use parser::binary::Result as ParseResult;


/// Generic FBX node.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct GenericNode {
    /// Node name.
    pub name: String,
    /// Node attributes.
    pub attributes: Vec<OwnedAttribute>,
    /// Child nodes.
    pub children: Vec<GenericNode>,
}

impl GenericNode {
    /// Creates a new `GenericNode`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Loads all sibling nodes from the given parser.
    ///
    /// This reads N `StartNode` and N+1 { `EndNode` or `EndFbx` }.
    pub fn load_from_parser<R, P>(parser: &mut P,)
        -> ParseResult<(Vec<GenericNode>, Option<FbxFooter>)>
        where R: ParserSource,
              P: Parser<R>
    {
        let mut nodes = Vec::new();
        let mut footer = None;
        loop {
            let (name, attrs) = match parser.next_event()? {
                Event::StartFbx(_) => continue,
                Event::EndFbx(f) => {
                    footer = f.ok();
                    break;
                },
                Event::EndNode => break,
                Event::StartNode(node) => {
                    let name = node.name.to_owned();
                    let attrs = OwnedAttribute::load_attrs_from_parser_event(node.attributes)?;
                    (name, attrs)
                },
            };
            let children = GenericNode::load_from_parser(&mut parser.subtree_parser())?
                .0;
            let node = GenericNode {
                name: name,
                attributes: attrs,
                children: children,
            };
            nodes.push(node);
        }
        nodes.shrink_to_fit();
        Ok((nodes, footer))
    }
}


/// Owned node attribute.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OwnedAttribute {
    /// `bool`.
    Bool(bool),
    /// `i16`.
    I16(i16),
    /// `i32`.
    I32(i32),
    /// `i64`.
    I64(i64),
    /// `f32`.
    F32(f32),
    /// `f64`.
    F64(f64),
    /// `Box<[bool]>`.
    ArrBool(Box<[bool]>),
    /// `Box<[i32]>`.
    ArrI32(Box<[i32]>),
    /// `Box<[i64]>`.
    ArrI64(Box<[i64]>),
    /// `Box<[f32]>`.
    ArrF32(Box<[f32]>),
    /// `Box<[f64]>`.
    ArrF64(Box<[f64]>),
    /// `String`.
    String(::std::result::Result<String, Vec<u8>>),
    /// `Box<[u8]>`.
    Binary(Box<[u8]>),
}

impl OwnedAttribute {
    /// Loads `OwnedAttribute`s from `parser::binary::Attributes`.
    pub fn load_attrs_from_parser_event<R>(mut attrs: Attributes<R>) -> ParseResult<Vec<Self>>
        where R: ParserSource
    {
        let mut result = Vec::with_capacity(attrs.num_attributes() as usize);
        while let Some(attr) = attrs.next_attribute()? {
            result.push(Self::load_from_parser_event(attr)?);
        }
        Ok(result)
    }

    /// Loads an `OwnedAttribute` from `parser::binary::Attribute`.
    pub fn load_from_parser_event<R>(attr: Attribute<R>) -> ::std::io::Result<Self>
        where R: ParserSource
    {
        use parser::binary::{PrimitiveAttribute, ArrayAttribute, SpecialAttributeType};
        Ok(match attr {
               Attribute::Primitive(PrimitiveAttribute::Bool(v)) => OwnedAttribute::Bool(v),
               Attribute::Primitive(PrimitiveAttribute::I16(v)) => OwnedAttribute::I16(v),
               Attribute::Primitive(PrimitiveAttribute::I32(v)) => OwnedAttribute::I32(v),
               Attribute::Primitive(PrimitiveAttribute::I64(v)) => OwnedAttribute::I64(v),
               Attribute::Primitive(PrimitiveAttribute::F32(v)) => OwnedAttribute::F32(v),
               Attribute::Primitive(PrimitiveAttribute::F64(v)) => OwnedAttribute::F64(v),
               Attribute::Array(ArrayAttribute::Bool(arr)) => {
                   OwnedAttribute::ArrBool(arr.into_vec()?.into_boxed_slice())
               },
               Attribute::Array(ArrayAttribute::I32(arr)) => {
                   OwnedAttribute::ArrI32(arr.into_vec()?.into_boxed_slice())
               },
               Attribute::Array(ArrayAttribute::I64(arr)) => {
                   OwnedAttribute::ArrI64(arr.into_vec()?.into_boxed_slice())
               },
               Attribute::Array(ArrayAttribute::F32(arr)) => {
                   OwnedAttribute::ArrF32(arr.into_vec()?.into_boxed_slice())
               },
               Attribute::Array(ArrayAttribute::F64(arr)) => {
                   OwnedAttribute::ArrF64(arr.into_vec()?.into_boxed_slice())
               },
               Attribute::Special(v) => {
                   match v.value_type() {
                       SpecialAttributeType::Binary => {
                           OwnedAttribute::Binary(v.into_vec()?.into_boxed_slice())
                       },
                       SpecialAttributeType::String => {
                           OwnedAttribute::String(match String::from_utf8(v.into_vec()?) {
                                                      Ok(s) => Ok(s),
                                                      Err(e) => Err(e.into_bytes()),
                                                  })
                       },
                   }
               },
           })
    }
}
