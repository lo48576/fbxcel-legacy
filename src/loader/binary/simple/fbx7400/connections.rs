//! `Connections` node and its children.

use parser::binary::{Parser, ParserSource, Attributes};
use loader::binary::simple::{Result, Error};


/// `Connections` node.
#[derive(Debug, Clone, PartialEq)]
pub struct Connections(pub Vec<Connection>);

impl Connections {
    /// Loads node contents from the parser.
    pub fn load<R, P>(mut parser: P) -> Result<Self>
    where
        R: ParserSource,
        P: Parser<R>,
    {
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
    pub source_id: i64,
    pub destination_id: i64,
    pub property: Option<String>,
    pub source_is_prop: bool,
    pub destination_is_prop: bool,
}

impl ConnectionAttrs {
    /// Loads attributes.
    pub fn load<R>(name: &str, mut attrs: Attributes<R>) -> Result<Self>
    where
        R: ParserSource,
    {
        use parser::binary::utils::AttributeValues;

        if name == "C" {
            let (ty, source_id, destination_id) =
                <(String, i64, i64)>::from_attributes(&mut attrs)?
                    .ok_or_else(|| Error::InvalidAttribute("C".to_owned()))?;
            let (source_is_prop, destination_is_prop) = match ty.as_str() {
                "OO" => (false, false),
                "OP" => (false, true),
                "PO" => (true, false),
                "PP" => (true, true),
                _ => return Err(Error::InvalidAttribute("C".to_owned())),
            };
            let property = if attrs.rest_attributes() > 0 {
                Some(String::from_attributes(&mut attrs)?.ok_or_else(|| {
                    Error::InvalidAttribute("C".to_owned())
                })?)
            } else {
                None
            };
            Ok(ConnectionAttrs {
                source_id: source_id,
                destination_id: destination_id,
                property: property,
                source_is_prop: source_is_prop,
                destination_is_prop: destination_is_prop,
            })
        } else {
            Err(Error::UnexpectedNode(name.to_owned()))
        }
    }
}


/// `C` node.
///
/// Note that "the child node will be a **source object** of the parent node", and "the parent node
/// will be the **destination object** of the child node".
/// See [FBX 2018 Developer Help:
/// Connections](https://help.autodesk.com/view/FBX/2018/ENU/?guid=__files_GUID_BB63A93A_7663_4256_B060_8EA35CB0FF3A_htm)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connection {
    /// Object ID of the source object.
    pub source: i64,
    /// Object ID of the destination object.
    pub destination: i64,
    /// Property of the connection.
    pub property: Option<String>,
    /// `true` if the source is property.
    pub source_is_prop: bool,
    /// `true` if the destination is property.
    pub destination_is_prop: bool,
}

impl Connection {
    /// Loads node contents from the parser.
    fn load<R, P>(mut parser: P, attrs: ConnectionAttrs) -> Result<Self>
    where
        R: ParserSource,
        P: Parser<R>,
    {
        parser.skip_current_node()?;
        Ok(Connection {
            source: attrs.source_id,
            destination: attrs.destination_id,
            property: attrs.property,
            source_is_prop: attrs.source_is_prop,
            destination_is_prop: attrs.destination_is_prop,
        })
    }
}
