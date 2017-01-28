//! `Objects` node and its children.

use fnv::FnvHashMap;
use parser::binary::{Parser, ParserSource, Attributes};
use loader::binary::simple::{Result, Error, GenericNode};
use loader::binary::simple::fbx7400::separate_name_class;


/// Map type with key = `i64`.
pub type ObjectMap<T> = FnvHashMap<i64, T>;


/// Properties common to object nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperties {
    /// ID.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Class.
    pub class: String,
    /// Subclass.
    pub subclass: String,
}

impl ::parser::binary::utils::AttributeValues for ObjectProperties {
    fn from_attributes<R: ParserSource>(attrs: &mut Attributes<R>)
        -> ::parser::binary::Result<Option<Self>> {
        let (id, name_class, subclass) =
            match <(i64, String, String)>::from_attributes(attrs)? {
                Some(v) => v,
                None => return Ok(None),
            };
        Ok(separate_name_class(&name_class).map(|(name, class)| {
            ObjectProperties {
                id: id,
                name: name.to_owned(),
                class: class.to_owned(),
                subclass: subclass,
            }
        }))
    }
}


/// `Objects`.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Objects {
    /// Unknown type.
    pub unknown: ObjectMap<UnknownObject>,
}

impl Objects {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        use parser::binary::utils::AttributeValues;

        let mut objects: Objects = Default::default();

        loop {
            use parser::binary::Event;
            let obj_props = match parser.next_event()? {
                Event::StartNode(mut info) => {
                    <ObjectProperties>::from_attributes(&mut info.attributes)?
                        .ok_or_else(|| Error::InvalidAttribute(info.name.to_owned()))?
                },
                Event::EndNode => break,
                ev => panic!("Unexpected node event: {:?}", ev),
            };
            // Node name is inferable from object class, therefor the code here only requires
            // object properties to decide node type.
            match (obj_props.class.as_str(), obj_props.subclass.as_str()) {
                _ => {
                    warn!("Unknown object type: {:?}", obj_props);
                    let id = obj_props.id;
                    let obj = UnknownObject::load(parser.subtree_parser(), obj_props)?;
                    objects.unknown.insert(id, obj);
                },
            }
        }
        Ok(objects)
    }
}


/// An object node of unknown type.
#[derive(Debug, Clone, PartialEq)]
pub struct UnknownObject {
    /// Object properties.
    pub properties: ObjectProperties,
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl UnknownObject {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(
        mut parser: P,
        attrs: ObjectProperties
    ) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(UnknownObject {
            properties: attrs,
            nodes: nodes,
        })
    }
}
