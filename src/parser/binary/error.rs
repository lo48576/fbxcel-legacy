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
    /// Successfully finished parsing FBX data.
    Finished,
    /// Magic binary not detected.
    MagicNotDetected([u8; 21]),
    /// Node name has invalid UTF-8 sequences.
    NodeNameInvalidUtf8(Arc<string::FromUtf8Error>),
    /// I/O error.
    Io(io::Error),
    /// End offset of a node is wrong.
    WrongNodeEndOffset {
        /// Start offset of the node.
        begin: u64,
        /// End offset of the node told by node header.
        expected_end: u64,
        /// Position of the node detected while reading input.
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
            Error::MagicNotDetected(ref bytes) => {
                write!(f, "Magic binary not detected: Got {:?}", bytes)
            },
            Error::NodeNameInvalidUtf8(ref err) => {
                write!(f, "Node name is not vaiid UTF-8 string: {}", err)
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
            Error::Finished => "Successfully finished parsing and there are no more data",
            Error::MagicNotDetected(_) => "Magic binary not detected",
            Error::NodeNameInvalidUtf8(_) => "Node name is not vaiid UTF-8 string",
            Error::Io(ref err) => err.description(),
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
            Error::Io(ref err) => {
                // To clone `io::Error`, convert inner error into string and use it as new inner error.
                Error::Io(io::Error::new(err.kind(), error::Error::description(err)))
            },
            ref err => err.clone(),
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
    /// Unknown 2 bytes right after FBX magic is unexpected.
    UnexpectedBytesAfterMagic([u8; 2]),
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
            Warning::UnexpectedBytesAfterMagic(_) => "Unexpected bytes right after magic binary",
        }
    }
}
