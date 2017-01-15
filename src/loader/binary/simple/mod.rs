//! Simple binary loader.

use parser::binary::{Parser, ParserSource, Event};
pub use self::error::{Result, Error};
pub use self::generic::{GenericNode, OwnedAttribute};

pub mod error;
pub mod generic;
pub mod fbx7400;


/// FBX tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Fbx {
    /// FBX 7.4 or later.
    Fbx7400(fbx7400::Fbx7400),
}

impl Fbx {
    /// Loads FBX structure from the given parser.
    pub fn load_from_parser<R: ParserSource, P: Parser<R>>(mut parser: P) -> Result<Self> {
        let version = match parser.next_event()? {
            Event::StartFbx(header) => header.version,
            ev => {
                panic!("FBX binary parser should return `StartFbx` as the first event but got \
                        `{:?}`",
                       ev)
            },
        };
        match version {
            7400...7599 => fbx7400::Fbx7400::load_from_parser(version, parser).map(Fbx::Fbx7400),
            _ => {
                error!("Unsupported FBX version: {}", version);
                unimplemented!()
            },
        }
    }
}
