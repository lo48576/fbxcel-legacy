//! Objects.

use parser::binary::{Parser, ParserSource, Attributes, SubtreeParser};
use parser::binary::Error as ParseError;
use loader::binary::simple::Result;
use loader::binary::simple::fbx7400::separate_name_class;


/// A trait for objects nodes loader of FBX 7.4 compatible data.
pub trait LoadObjects7400: Sized {
    /// Result objects.
    type Objects;

    /// Build objects from the loader.
    fn build(self) -> Result<Self::Objects>;

    /// Load an object.
    fn load<R: ParserSource>(
        &mut self,
        props: ObjectProperties,
        subtree_parser: &mut SubtreeParser<R>
    ) -> Result<()>;
}


/// Loads node contents from the parser.
pub fn load<R: ParserSource, P: Parser<R>, O: LoadObjects7400>(
    mut parser: P,
    mut objs_loader: O
) -> Result<O::Objects> {
    loop {
        let props = try_get_node_attrs!(parser, ObjectProperties::load);
        let mut sub_parser = parser.subtree_parser();
        objs_loader.load(props, &mut sub_parser)?;
        sub_parser.skip_current_node()?;
    }
    objs_loader.build()
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
    fn load<R: ParserSource>(name: &str, mut attrs: Attributes<R>) -> Result<ObjectProperties> {
        use parser::binary::utils::AttributeValues;
        use loader::binary::simple::Error;

        Self::from_attributes(&mut attrs)?.ok_or_else(|| Error::InvalidAttribute(name.to_owned()))
    }
}

impl ::parser::binary::utils::AttributeValues for ObjectProperties {
    fn from_attributes<R: ParserSource>(attrs: &mut Attributes<R>)
        -> ::std::result::Result<Option<Self>, ParseError> {
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
