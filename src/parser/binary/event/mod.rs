//! Parser event.

use std::io;

use parser::binary::RootParser;
use parser::binary::error::{Result, Error, Warning};
use parser::binary::reader::{ParserSource, ReadLittleEndian};
pub use self::attribute::{Attributes, Attribute, SpecialAttributeType};
pub use self::attribute::{PrimitiveAttribute, ArrayAttribute, SpecialAttribute};
pub use self::attribute::ArrayAttributeReader;

mod attribute;


/// Parser event.
#[derive(Debug)]
pub enum Event<'a, R: 'a> {
    /// Start of the FBX document.
    StartFbx(FbxHeader),
    /// End of the FBX document.
    EndFbx(Result<FbxFooter>),
    /// Start of a node.
    StartNode(StartNode<'a, R>),
    /// End of a node.
    EndNode,
}

impl<'a, R: 'a + ParserSource> From<FbxHeader> for Event<'a, R> {
    fn from(h: FbxHeader) -> Self {
        Event::StartFbx(h)
    }
}

impl<'a, R: 'a + ParserSource> From<Result<FbxFooter>> for Event<'a, R> {
    fn from(f: Result<FbxFooter>) -> Self {
        Event::EndFbx(f)
    }
}

impl<'a, R: 'a + ParserSource> From<StartNode<'a, R>> for Event<'a, R> {
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
pub fn read_fbx_header<R>(parser: &mut RootParser<R>) -> Result<FbxHeader>
    where R: ParserSource
{
    assert!(parser.fbx_version.is_none(),
            "Parser should read FBX header only once");
    // Check magic binary.
    {
        const MAGIC_LEN: usize = 21;
        const MAGIC: &'static [u8; MAGIC_LEN] = b"Kaydara FBX Binary  \x00";
        let mut buf = [0u8; MAGIC_LEN];
        parser.source.read_exact(&mut buf)?;
        if buf != *MAGIC {
            return Err(Error::MagicNotDetected(buf));
        }
    }
    // Read unknown 2 bytes.
    {
        const UNKNOWN_BYTES_LEN: usize = 2;
        const UNKNOWN_BYTES: &'static [u8; UNKNOWN_BYTES_LEN] = b"\x1a\x00";
        let mut buf = [0u8; UNKNOWN_BYTES_LEN];
        parser.source.read_exact(&mut buf)?;
        if buf != *UNKNOWN_BYTES {
            parser.warn(Warning::UnexpectedBytesAfterMagic(buf));
        }
    }
    // Get FBX version.
    let fbx_version = parser.source.read_u32()?;

    info!("FBX header is successfully read, FBX version: {}",
          fbx_version);
    Ok(FbxHeader { version: fbx_version })
}


/// FBX footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FbxFooter {
    /// Unknown part 1.
    pub unknown1: [u8; 16],
    /// FBX version.
    pub version: u32,
    /// Unknown part 2.
    pub unknown2: [u8; 16],
}

impl FbxFooter {
    /// Reads node header from the given parser and returns it.
    pub fn read_from_parser<R>(parser: &mut RootParser<R>) -> Result<Self>
        where R: ParserSource
    {
        // Read unknown 16 bytes footer.
        let mut unknown1 = [0u8; 16];
        parser.source.read_exact(&mut unknown1)?;
        // Read padding (0--15 bytes), zeroes (4 bytes), FBX version (4 bytes), zeroes (120 bytes),
        // and optionally partial unknown footer 2 (16 bytes).
        // Note that some exporters (like Blender's "FBX format" plugin version 3.2.0) creates
        // wrong FBX file without padding.
        // For such file without padding, unknown footer 2 would be partially read.
        let expected_padding_len = ((16 - (parser.source.position() & 0x0f)) & 0x0f) as usize;
        debug!("Current position = {}, Expected padding length = {}",
               parser.source.position(),
               expected_padding_len);

        const BUF_LEN: usize = 144;
        let mut buf = [0u8; BUF_LEN];
        parser.source.read_exact(&mut buf)?;
        // If there is no padding before the footer, unknown footer 2 is partially read into the
        // buf.
        // Count length of partially read unknown footer 2.
        let partial_footer2_len = {
            let mut count = 0;
            // Unknown footer 2 doesn't contain 0x00 byte, therefore the last 0x00 should be
            // the last byte of a padding.
            while (buf[BUF_LEN - 1 - count] != 0) && count <= 16 {
                count += 1;
            }
            if count > 16 {
                error!("FBX footer should have continuous 112 bytes of zeroes, but not found");
                return Err(Error::BrokenFbxFooter);
            }
            count
        };
        let mut unknown2 = [0u8; 16];
        // Copy partially read unknown header 2.
        unknown2[0..partial_footer2_len].clone_from_slice(&buf[BUF_LEN - partial_footer2_len..
                                                           BUF_LEN]);
        // Read the rest of the unknown footer 2 (max 16 bytes).
        parser.source.read_exact(&mut unknown2[partial_footer2_len..])?;

        // Check whether padding before the footer exists.
        if 16 - partial_footer2_len == expected_padding_len {
            // Padding exists.
            // Note that its length might be 0.
            info!("Padding exists (as expected) before the footer (len={})",
                  expected_padding_len);
        } else {
            parser.warn(Warning::InvalidPaddingInFbxFooter {
                            expected: expected_padding_len as u8,
                            actual: 16 - partial_footer2_len as u8,
                        });
        }

        // Check the FBX version.
        let footer_fbx_version = {
            // 20 - partial_footer2_len == BUF_LEN - partial_footer2_len - 120 - 4
            let ver_offset = 20 - partial_footer2_len;
            // FBX version is stored as `u32` in Little Endian.
            (buf[ver_offset] as u32) | (buf[ver_offset + 1] as u32) << 8 |
            (buf[ver_offset + 2] as u32) << 16 | (buf[ver_offset + 3] as u32) << 24
        };
        let header_fbx_version =
            parser.fbx_version
                .expect("Parser should remember FBX version in the FBX header but it doesn't");
        if header_fbx_version != footer_fbx_version {
            return Err(Error::HeaderFooterVersionMismatch {
                           header: header_fbx_version,
                           footer: footer_fbx_version,
                       });
        }

        Ok(FbxFooter {
               unknown1: unknown1,
               version: footer_fbx_version,
               unknown2: unknown2,
           })
    }
}

/// FBX node info.
#[derive(Debug)]
pub struct StartNode<'a, R: 'a> {
    /// Node name.
    pub name: &'a str,
    /// Node attributes.
    pub attributes: Attributes<'a, R>,
}


/// Parser event without reference to a parser.
#[derive(Debug, Clone)]
pub enum EventBuilder {
    /// Start of the FBX document.
    StartFbx(FbxHeader),
    /// End of the FBX document.
    EndFbx(Result<FbxFooter>),
    /// Start of a node.
    StartNode(StartNodeBuilder),
    /// End of a node.
    EndNode,
}

impl EventBuilder {
    /// Creates `Event` from the `EventBuilder` and the given parser.
    pub fn build<R>(self, parser: &mut RootParser<R>) -> Event<R>
        where R: ParserSource
    {
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

impl From<Result<FbxFooter>> for EventBuilder {
    fn from(f: Result<FbxFooter>) -> Self {
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
    /// Node header.
    pub header: NodeHeader,
}

impl StartNodeBuilder {
    /// Creates `StartNode` from the `StartNodeBuilder` and the given parser.
    pub fn build<R>(self, parser: &mut RootParser<R>) -> StartNode<R>
        where R: ParserSource
    {
        let RootParser {
            ref mut source,
            ref mut warnings,
            ref recent_node_name,
            ..
        } = *parser;
        StartNode {
            name:
                recent_node_name.as_ref().expect("`RootParser::recent_node_name` must not be empty"),
            attributes: attribute::new_attributes(source, warnings, &self.header),
        }
    }
}


/// Fixed size node header (without node name field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeHeader {
    /// End offset of the node.
    pub end_offset: u64,
    /// Number of the node attributes.
    pub num_attributes: u64,
    /// Length of the node attributes in bytes.
    pub bytelen_attributes: u64,
    /// Length of the node name in bytes.
    pub bytelen_name: u8,
}

impl NodeHeader {
    /// Returns true if all fields of the node header is `0`.
    pub fn is_node_end(&self) -> bool {
        self.end_offset == 0 && self.num_attributes == 0 && self.bytelen_attributes == 0 &&
        self.bytelen_name == 0
    }

    /// Reads node header from the given parser and returns it.
    pub fn read_from_parser<R>(parser: &mut RootParser<R>) -> io::Result<Self>
        where R: ParserSource
    {
        let fbx_version =
            parser.fbx_version
                .expect("Attempt to read FBX node header but the parser doesn't know FBX version");
        let (end_offset, num_attributes, bytelen_attributes) = if fbx_version < 7500 {
            let eo = parser.source.read_u32()? as u64;
            let na = parser.source.read_u32()? as u64;
            let bla = parser.source.read_u32()? as u64;
            (eo, na, bla)
        } else {
            let eo = parser.source.read_u64()?;
            let na = parser.source.read_u64()?;
            let bla = parser.source.read_u64()?;
            (eo, na, bla)
        };
        let bytelen_name = parser.source.read_u8()?;
        Ok(NodeHeader {
               end_offset: end_offset,
               num_attributes: num_attributes,
               bytelen_attributes: bytelen_attributes,
               bytelen_name: bytelen_name,
           })
    }
}
