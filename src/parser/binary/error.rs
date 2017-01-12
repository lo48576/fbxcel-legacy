//! FBX parsing error.

use std::error;
use std::fmt;
use std::io;
use std::string;
use std::sync::Arc;


/// Result of parser function.
pub type Result<T> = ::std::result::Result<T, Error>;


/// FBX parsing error.
#[derive(Debug)]
pub enum Error {
    /// FBX footer is broken.
    BrokenFbxFooter,
    /// Successfully finished parsing the target FBX node(s).
    Finished,
    /// Specified FBX versions mismatched in header and footer.
    HeaderFooterVersionMismatch {
        /// Version specified in header.
        header: u32,
        /// Version specified in footer.
        footer: u32,
    },
    /// Invalid node attribute type code.
    InvalidNodeAttributeTypeCode {
        /// Got type code.
        got: u8,
        /// Position of the type code.
        position: u64,
    },
    /// Magic binary not detected.
    MagicNotDetected([u8; 21]),
    /// Node name has invalid UTF-8 sequences.
    NodeNameInvalidUtf8(Arc<string::FromUtf8Error>),
    /// I/O error.
    Io(io::Error),
    /// Unknown array attribute encoding.
    UnknownArrayAttributeEncoding(u32),
    /// End offset of a node is wrong.
    WrongNodeEndOffset {
        /// Start offset of the node.
        begin: u64,
        /// End offset of the node told by node header.
        expected_end: u64,
        /// Position of the end of the node detected while reading input.
        real_end: u64,
    },
}

impl Error {
    /// Creates `Error:NodeNameInvalidUtf8(_)` from the given error.
    pub fn node_name_invalid_utf8(e: string::FromUtf8Error) -> Self {
        Error::NodeNameInvalidUtf8(Arc::new(e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::HeaderFooterVersionMismatch { header, footer } => {
                write!(f,
                       "FBX version is {} in the FBX header but {} in the footer",
                       header,
                       footer)
            },
            Error::InvalidNodeAttributeTypeCode { got, position } => {
                write!(f,
                       "Invalid node attribute type code: Got {:?} at position {}",
                       got,
                       position)
            },
            Error::MagicNotDetected(ref bytes) => {
                write!(f, "Magic binary not detected: Got {:?}", bytes)
            },
            Error::NodeNameInvalidUtf8(ref err) => {
                write!(f, "Node name is not vaiid UTF-8 string: {}", err)
            },
            Error::UnknownArrayAttributeEncoding(val) => {
                write!(f, "Unknown array attribute encoding: encoding={}", val)
            },
            Error::WrongNodeEndOffset { begin, expected_end, real_end } => {
                write!(f,
                       "Node ends with unexpected position: begin={}, expected_end={}, real_end={}",
                       begin,
                       expected_end,
                       real_end)
            },
            _ => write!(f, "{}", (self as &error::Error).description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BrokenFbxFooter => "FBX footer is broken",
            Error::Finished => "Successfully finished parsing and there are no more data",
            Error::HeaderFooterVersionMismatch { .. } => {
                "Specified FBX versions mismatched in header and footer"
            },
            Error::InvalidNodeAttributeTypeCode { .. } => "Invalid node attribute type code",
            Error::MagicNotDetected(_) => "Magic binary not detected",
            Error::NodeNameInvalidUtf8(_) => "Node name is not vaiid UTF-8 string",
            Error::Io(ref err) => err.description(),
            Error::UnknownArrayAttributeEncoding(_) => "Unknown array attribute encoding",
            Error::WrongNodeEndOffset { .. } => "Wrong node end offset",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::NodeNameInvalidUtf8(ref err) => Some(&**err),
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match *self {
            Error::BrokenFbxFooter => Error::BrokenFbxFooter,
            Error::Finished => Error::Finished,
            Error::InvalidNodeAttributeTypeCode { got, position } => {
                Error::InvalidNodeAttributeTypeCode {
                    got: got,
                    position: position,
                }
            },
            Error::HeaderFooterVersionMismatch { header, footer } => {
                Error::HeaderFooterVersionMismatch {
                    header: header,
                    footer: footer,
                }
            },
            Error::MagicNotDetected(v) => Error::MagicNotDetected(v),
            Error::NodeNameInvalidUtf8(ref err) => Error::NodeNameInvalidUtf8(err.clone()),
            Error::Io(ref err) => {
                // To clone `io::Error`, convert inner error into string and use it
                // as a new inner error.
                Error::Io(io::Error::new(err.kind(), error::Error::description(err)))
            },
            Error::UnknownArrayAttributeEncoding(v) => Error::UnknownArrayAttributeEncoding(v),
            Error::WrongNodeEndOffset { begin, expected_end, real_end } => {
                Error::WrongNodeEndOffset {
                    begin: begin,
                    expected_end: expected_end,
                    real_end: real_end,
                }
            },
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// FBX parser warning.
#[derive(Debug, Clone, Copy)]
pub enum Warning {
    /// Invalid node attribute of boolean value.
    InvalidBooleanAttributeValue {
        /// Got value.
        got: u8,
        /// Assumed value.
        assumed: bool,
        /// Position of the attribute value.
        position: u64,
    },
    /// FBX footer has invalid padding.
    InvalidPaddingInFbxFooter {
        /// Expected padding length.
        expected: u8,
        /// Actual padding length.
        actual: u8,
    },
    /// Unknown 2 bytes right after FBX magic is unexpected.
    UnexpectedBytesAfterMagic([u8; 2]),
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Warning::InvalidBooleanAttributeValue { got, assumed, position } => {
                write!(f,
                       "Invalid boolean node attribute value at position {}: got {:?}, assumed {}",
                       position,
                       got,
                       assumed)
            },
            Warning::InvalidPaddingInFbxFooter { expected, actual } => {
                write!(f,
                       "Invalid padding in FBX footer: expected {} bytes but got {} bytes",
                       expected,
                       actual)
            },
            Warning::UnexpectedBytesAfterMagic(ref bytes) => {
                write!(f,
                       "Unexpected bytes right after magic binary: expected [0x1a, 0x00] but got \
                        {:?}",
                       bytes)
            },
        }
    }
}

impl error::Error for Warning {
    fn description(&self) -> &str {
        match *self {
            Warning::InvalidBooleanAttributeValue { .. } => "Invalid boolean node attribute value",
            Warning::InvalidPaddingInFbxFooter { .. } => "Invalid padding in FBX footer",
            Warning::UnexpectedBytesAfterMagic(_) => "Unexpected bytes right after magic binary",
        }
    }
}
