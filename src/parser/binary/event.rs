//! Parser event.

use std::io::Read;

use super::BinaryParser;
use super::error::{Result, Error, Warning};


/// Parser event.
#[derive(Debug)]
pub enum Event<'a, R: 'a + Read> {
    /// Start of the FBX document.
    StartFbx(FbxHeader),
    /// End of the FBX document.
    EndFbx(FbxFooter),
    /// Start of a node.
    StartNode(StartNode<'a, R>),
    /// End of a node.
    EndNode,
}

impl<'a, R: 'a + Read> From<FbxHeader> for Event<'a, R> {
    fn from(h: FbxHeader) -> Self {
        Event::StartFbx(h)
    }
}

impl<'a, R: 'a + Read> From<FbxFooter> for Event<'a, R> {
    fn from(f: FbxFooter) -> Self {
        Event::EndFbx(f)
    }
}

impl<'a, R: 'a + Read> From<StartNode<'a, R>> for Event<'a, R> {
    fn from(h: StartNode<'a, R>) -> Self {
        Event::StartNode(h)
    }
}


/// FBX header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FbxHeader {
}


/// FBX footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FbxFooter {
}


/// FBX node info.
#[derive(Debug)]
pub struct StartNode<'a, R: 'a + Read> {
    /// Parser.
    _parser: &'a mut BinaryParser<R>,
}


/// Parser event without reference to a parser.
#[derive(Debug, Clone)]
pub enum EventBuilder {
    /// Start of the FBX document.
    StartFbx(FbxHeader),
    /// End of the FBX document.
    EndFbx(FbxFooter),
    /// Start of a node.
    StartNode(StartNodeBuilder),
    /// End of a node.
    EndNode,
}

impl EventBuilder {
    /// Creates `Event` from the `EventBuilder` and the given parser.
    pub fn build<R: Read>(self, parser: &mut BinaryParser<R>) -> Event<R> {
        match self {
            EventBuilder::StartFbx(header) => header.into(),
            EventBuilder::EndFbx(footer) => footer.into(),
            EventBuilder::StartNode(builder) => builder.build(parser).into(),
            EventBuilder::EndNode => Event::EndNode,
        }
    }
}

impl From<FbxHeader> for EventBuilder {
    fn from(h: FbxHeader) -> Self {
        EventBuilder::StartFbx(h)
    }
}

impl From<FbxFooter> for EventBuilder {
    fn from(f: FbxFooter) -> Self {
        EventBuilder::EndFbx(f)
    }
}

impl From<StartNodeBuilder> for EventBuilder {
    fn from(h: StartNodeBuilder) -> Self {
        EventBuilder::StartNode(h)
    }
}


/// `StartNode` without reference to a parser.
#[derive(Debug, Clone)]
pub struct StartNodeBuilder {
}

impl StartNodeBuilder {
    /// Creates `StartNode` from the `StartNodeBuilder` and the given parser.
    pub fn build<R: Read>(self, parser: &mut BinaryParser<R>) -> StartNode<R> {
        StartNode { _parser: parser }
    }
}
