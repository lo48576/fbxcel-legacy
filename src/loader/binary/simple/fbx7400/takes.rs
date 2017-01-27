//! `Takes` node and its children.

use parser::binary::{Parser, ParserSource, Event};
use loader::binary::simple::Result;


/// `Takes` node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Takes {
    /// `Current`.
    pub current: String,
    /// `Take`s.
    pub takes: Vec<Take>,
}

impl Takes {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let mut current = None;
        let mut takes = Vec::new();

        loop {
            let node_type = try_get_node_attrs!(parser, TakesChildAttrs::load);
            match node_type {
                TakesChildAttrs::Current(v) => {
                    current = Some(v);
                    parser.skip_current_node()?;
                },
                TakesChildAttrs::Take(attrs) => {
                    takes.push(Take::load(parser.subtree_parser(), attrs)?);
                },
            }
        }
        Ok(Takes {
            current: ensure_node_exists!(current, "Current"),
            takes: takes,
        })
    }
}


child_attr_loader! { TakesChildAttrs {
    "Current" => Current(String),
    "Take" => Take(String),
}}


/// `Take` node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Take {
    /// Name.
    pub name: String,
    /// `FileName`.
    pub filename: String,
    /// `LocalTime`.
    pub local_time: (i64, i64),
    /// `ReferenceTime`.
    pub reference_time: (i64, i64),
}

impl Take {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P, attrs: String) -> Result<Self> {
        let mut filename = None;
        let mut local_time = None;
        let mut reference_time = None;

        loop {
            let node_type = try_get_node_attrs!(parser, TakeChildAttrs::load);
            match node_type {
                TakeChildAttrs::FileName(v) => {
                    filename = Some(v);
                },
                TakeChildAttrs::LocalTime(v) => {
                    local_time = Some(v);
                },
                TakeChildAttrs::ReferenceTime(v) => {
                    reference_time = Some(v);
                },
            }
            parser.skip_current_node()?;
        }
        Ok(Take {
            name: attrs,
            filename: ensure_node_exists!(filename, "Take"),
            local_time: ensure_node_exists!(local_time, "Take"),
            reference_time: ensure_node_exists!(reference_time, "Take"),
        })
    }
}


child_attr_loader! { TakeChildAttrs {
    "FileName" => FileName(String),
    "LocalTime" => LocalTime((i64, i64)),
    "ReferenceTime" => ReferenceTime((i64, i64)),
}}
