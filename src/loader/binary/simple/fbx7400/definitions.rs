//! `Definitions` node and its children.

use fnv::FnvHashMap;
use parser::binary::{Parser, ParserSource, Attributes};
use loader::binary::simple::{Result, Error};
use loader::binary::simple::fbx7400::Properties70;


/// `Definitions` node.
#[derive(Debug, Clone, PartialEq)]
pub struct Definitions {
    /// Version of the node.
    pub version: i32,
    /// Reference count?
    pub count: i32,
    /// Property templates for object types.
    pub object_types: Vec<ObjectType>,
}

impl Definitions {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut version = None;
        let mut count = None;
        let mut object_types = Vec::new();

        loop {
            let node_type = try_get_node_attrs!(parser, DefinitionsChildAttrs::load);
            match node_type {
                DefinitionsChildAttrs::Version(v) => {
                    version = Some(v);
                    parser.skip_current_node()?;
                },
                DefinitionsChildAttrs::Count(v) => {
                    count = Some(v);
                    parser.skip_current_node()?;
                },
                DefinitionsChildAttrs::ObjectType(attrs) => {
                    object_types.push(ObjectType::load(parser.subtree_parser(), attrs)?);
                },
            }
        }
        Ok(Definitions {
               version: ensure_node_exists!(version, "Definitions", "Version"),
               count: ensure_node_exists!(count, "Definitions", "Count"),
               object_types: object_types,
           })
    }
}


child_attr_loader! { DefinitionsChildAttrs {
    "Count" => Count(i32),
    "Version" => Version(i32),
    "ObjectType" => ObjectType(String),
}}


/// An object type and property template for it.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType {
    /// Target object type.
    pub object_type: String,
    /// Reference count?
    pub count: i32,
    /// Property templates.
    pub property_template: FnvHashMap<String, Properties70>,
}

impl ObjectType {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: String) -> Result<Self> {
        let mut count = None;
        let mut property_template = FnvHashMap::default();

        loop {
            let node_type = try_get_node_attrs!(parser, ObjectTypeChildAttrs::load);
            match node_type {
                ObjectTypeChildAttrs::Count(v) => {
                    count = Some(v);
                    parser.skip_current_node()?;
                },
                ObjectTypeChildAttrs::PropertyTemplate(attrs) => {
                    let props = load_property_template(parser.subtree_parser())?;
                    property_template.insert(attrs, props);
                },
            }
        }

        Ok(ObjectType {
               object_type: attrs,
               count: ensure_node_exists!(count, "ObjectType", "Count"),
               property_template: property_template,
           })
    }
}


child_attr_loader! { ObjectTypeChildAttrs {
    "Count" => Count(i32),
    "PropertyTemplate" => PropertyTemplate(String),
}}


fn load_property_template<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Properties70> {
    let mut props = None;

    loop {
        try_get_node_attrs!(parser, |name: &str, _| if name == "Properties70" {
            Ok(())
        } else {
            Err(Error::UnexpectedNode(name.to_owned()))
        });
        props = Some(Properties70::load(parser.subtree_parser())?);
    }
    Ok(ensure_node_exists!(props, "PropertyTemplate", "Properties70"))
}
