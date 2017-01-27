//! `Objects` node and its children.

use parser::binary::{Parser, ParserSource, Event, Attributes};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::separate_name_class;


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
pub struct Objects {}

impl Objects {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        use parser::binary::utils::AttributeValues;

        let objects: Objects = Default::default();

        loop {
            let obj_props = match parser.next_event()? {
                Event::StartNode(mut info) => {
                    <ObjectProperties>::from_attributes(&mut info.attributes)?
                        .ok_or_else(|| Error::InvalidAttribute(info.name.to_owned()))?
                },
                Event::EndNode => break,
                ev => panic!("Unexpected node event: {:?}", ev),
            };
            debug!("obj_props: {:?}", obj_props);
            parser.skip_current_node()?;
        }
        Ok(objects)
    }
}
