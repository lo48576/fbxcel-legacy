//! Objects.

use parser::binary::{ParserSource, Attributes, SubtreeParser};
use parser::binary::Error as ParseError;
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::NodesBeforeObjects;
use loader::binary::simple::fbx7400::separate_name_class;


/// A trait for objects nodes loader of FBX 7.4 compatible data.
pub trait LoadObjects7400: Sized {
    /// Result objects.
    type Objects;

    /// Build objects from the loader.
    fn build(self) -> Result<Self::Objects>;

    /// Load an object.
    fn load<R>(
        &mut self,
        props: ObjectProperties,
        subtree_parser: &mut SubtreeParser<R>,
        nodes_before_objects: &NodesBeforeObjects
    ) -> Result<()>
        where R: ParserSource;
}


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

impl ObjectProperties {
    /// Loads `ObjectProperties` in the same manner as usual child node attributes.
    pub fn load<R>(name: &str, mut attrs: Attributes<R>) -> Result<ObjectProperties>
        where R: ParserSource
    {
        use parser::binary::utils::AttributeValues;
        use loader::binary::simple::Error;

        Self::from_attributes(&mut attrs)?.ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
    }
}

impl ::parser::binary::utils::AttributeValues for ObjectProperties {
    fn from_attributes<R>(attrs: &mut Attributes<R>)
        -> ::std::result::Result<Option<Self>, ParseError>
        where R: ParserSource
    {
        let (id, name_class, subclass) = match <(i64, String, String)>::from_attributes(attrs)? {
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
