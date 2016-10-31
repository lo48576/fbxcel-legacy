//! Parser event.

use std::io;
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
    /// FBX version.
    pub version: u32,
}


/// Read FBX header.
pub fn read_fbx_header<R: Read>(parser: &mut BinaryParser<R>) -> Result<FbxHeader> {
    assert!(parser.fbx_version.is_none(),
            "Parser should read FBX header only once");
    // Check magic binary.
    {
        const MAGIC_LEN: usize = 21;
        const MAGIC: &'static [u8; MAGIC_LEN] = b"Kaydara FBX Binary  \x00";
        let mut buf = [0u8; MAGIC_LEN];
        try!(parser.source.read_exact(&mut buf));
        if buf != *MAGIC {
            return Err(Error::MagicNotDetected(buf));
        }
    }
    // Read unknown 2 bytes.
    {
        const UNKNOWN_BYTES_LEN: usize = 2;
        const UNKNOWN_BYTES: &'static [u8; UNKNOWN_BYTES_LEN] = b"\x1a\x00";
        let mut buf = [0u8; UNKNOWN_BYTES_LEN];
        try!(parser.source.read_exact(&mut buf));
        if buf != *UNKNOWN_BYTES {
            parser.warn(Warning::UnexpectedBytesAfterMagic(buf));
        }
    }
    // Get FBX version.
    let fbx_version = try!(parser.source.read_u32());

    info!("FBX header is successfully read, FBX version: {}",
          fbx_version);
    Ok(FbxHeader { version: fbx_version })
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


/// Fixed size node header (without node name field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeHeader {
    /// End offset of the node.
    pub end_offset: u64,
    /// Number of the node attributes.
    pub num_attributes: u64,
    /// Byte length of the node attributes.
    pub len_attributes: u64,
    /// Length of the node name.
    pub len_name: u8,
}

impl NodeHeader {
    /// Returns true if all fields of the node header is `0`.
    pub fn is_node_end(&self) -> bool {
        self.end_offset == 0 && self.num_attributes == 0 && self.len_attributes == 0 &&
        self.len_name == 0
    }

    /// Reads node header from the given parser and returns it.
    pub fn read_from_parser<R: Read>(parser: &mut BinaryParser<R>) -> io::Result<Self> {
        let fbx_version = parser.fbx_version
            .expect("Attempt to read FBX node header but the parser doesn't know FBX version");
        let (end_offset, num_attributes, len_attributes) = if fbx_version < 7500 {
            let eo = try!(parser.source.read_u32()) as u64;
            let na = try!(parser.source.read_u32()) as u64;
            let la = try!(parser.source.read_u32()) as u64;
            (eo, na, la)
        } else {
            let eo = try!(parser.source.read_u64());
            let na = try!(parser.source.read_u64());
            let la = try!(parser.source.read_u64());
            (eo, na, la)
        };
        let len_name = try!(parser.source.read_u8());
        Ok(NodeHeader {
            end_offset: end_offset,
            num_attributes: num_attributes,
            len_attributes: len_attributes,
            len_name: len_name,
        })
    }
}
