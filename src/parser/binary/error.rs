//! FBX parsing error.

use std::error;
use std::fmt;
use std::io;


/// Result of parser function.
pub type Result<T> = ::std::result::Result<T, Error>;


/// FBX parsing error.
#[derive(Debug)]
pub enum Error {
    /// Successfully finished parsing FBX data.
    Finished,
    /// Magic binary not detected.
    MagicNotDetected([u8; 21]),
    /// I/O error.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MagicNotDetected(ref bytes) => {
                write!(f, "Magic binary not detected: Got {:?}", bytes)
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
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
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
