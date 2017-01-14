//! Simple FBX 7.4 binary loader.

use parser::binary::{Result, Parser, ParserSource, FbxFooter};
use loader::binary::simple::GenericNode;


/// FBX 7.4 or later.
#[derive(Debug, Clone, PartialEq)]
pub struct Fbx7400 {
    /// FBX version.
    pub version: u32,
    /// Nodes.
    pub nodes: Vec<GenericNode>,
    /// FBX footer.
    pub footer: Option<FbxFooter>,
}

impl Fbx7400 {
    /// Loads FBX 7400 (or later) structure from the given parser.
    pub fn load_from_parser<R: ParserSource, P: Parser<R>>(
        version: u32,
        mut parser: P
    ) -> Result<Self> {
        info!("FBX version: {}, loading in FBX 7400 mode", version);

        let (nodes, footer) = GenericNode::load_from_parser(&mut parser)?;
        Ok(Fbx7400 {
            version: version,
            nodes: nodes,
            footer: footer,
        })
    }
}
