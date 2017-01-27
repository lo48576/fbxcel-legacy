//! `Objects` node and its children.

use parser::binary::{Parser, ParserSource};
use loader::binary::simple::{Result, GenericNode};


/// `Objects`.
#[derive(Debug, Clone, PartialEq)]
pub struct Objects {
    /// Child nodes.
    pub nodes: Vec<GenericNode>,
}

impl Objects {
    /// Loads node contents from the parser.
    pub fn load<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let nodes = GenericNode::load_from_parser(&mut parser)?.0;
        Ok(Objects { nodes: nodes })
    }
}
