//! Simple loader error.

use std::error;
use std::fmt;
use std::io;
use parser::binary::Error as ParseError;


/// Load result.
pub type Result<T> = ::std::result::Result<T, Error>;


/// Load error.
#[derive(Debug, Clone)]
pub enum Error {
    /// Parse error (including I/O error).
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => write!(f, "{}", (self as &error::Error).description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Parse(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Parse(ref err) => Some(err),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Parse(e.into())
    }
}
