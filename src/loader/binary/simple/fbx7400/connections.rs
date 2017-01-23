//! `Connections` node and its children.

use parser::binary::{Parser, ParserSource, Event, Attributes};
use loader::binary::simple::{Result, Error};


/// `Connections` node.
#[derive(Debug, Clone, PartialEq)]
pub struct Connections(pub Vec<Connection>);

impl Connections {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut connections = Vec::new();

        loop {
            let attrs = try_get_node_attrs!(parser, ConnectionAttrs::load);
            connections.push(Connection::load(parser.subtree_parser(), attrs)?);
        }
        Ok(Connections(connections))
    }
}


/// Attributes read from a `C` node.
struct ConnectionAttrs {
    pub child_id: i64,
    pub parent_id: i64,
    pub property: Option<String>,
    pub child_is_prop: bool,
    pub parent_is_prop: bool,
}

impl ConnectionAttrs {
    /// Loads attributes.
    pub fn load<R: ParserSource>(name: &str, mut attrs: Attributes<R>) -> Result<Self> {
        use parser::binary::utils::AttributeValues;

        if name == "C" {
            let (ty, child_id, parent_id) =
                <(String, i64, i64)>::from_attributes(&mut attrs)?
                    .ok_or_else(|| Error::InvalidAttribute("C".to_owned()))?;
            let (child_is_prop, parent_is_prop) = match ty.as_str() {
                "OO" => (false, false),
                "OP" => (false, true),
                "PO" => (true, false),
                "PP" => (true, true),
                _ => return Err(Error::InvalidAttribute("C".to_owned())),
            };
            let property = if attrs.rest_attributes() > 0 {
                Some(String::from_attributes(&mut attrs)?
                    .ok_or_else(|| Error::InvalidAttribute("C".to_owned()))?)
            } else {
                None
            };
            Ok(ConnectionAttrs {
                child_id: child_id,
                parent_id: parent_id,
                property: property,
                child_is_prop: child_is_prop,
                parent_is_prop: parent_is_prop,
            })
        } else {
            Err(Error::UnexpectedNode(name.to_owned()))
        }
    }
}


/// `C` node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connection {
    /// Object ID of the child object.
    pub child: i64,
    /// Object ID of the parent object.
    pub parent: i64,
    /// Property of the connection.
    pub property: Option<String>,
    /// `true` if the child is property.
    pub child_is_prop: bool,
    /// `true` if the parent is property.
    pub parent_is_prop: bool,
}

impl Connection {
    /// Loads node contents from the parser.
    fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: ConnectionAttrs) -> Result<Self> {
        parser.skip_current_node()?;
        Ok(Connection {
            child: attrs.child_id,
            parent: attrs.parent_id,
            property: attrs.property,
            child_is_prop: attrs.child_is_prop,
            parent_is_prop: attrs.parent_is_prop,
        })
    }
}
