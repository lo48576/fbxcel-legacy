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
    /// Inconsistent data.
    ///
    /// This means there's schema error among multiple nodes or attributes.
    Inconsistent(String),
    /// Attribute is invalid.
    ///
    /// This includes the cases below:
    /// - The node has too few or too many node attributes
    /// - The node has wrong types of node attribute values.
    /// - The node has wrong (or unsupported) values of node attributes values.
    InvalidAttribute(String),
    /// Required node is missing.
    MissingNode(String),
    /// Parse error (including I/O error).
    Parse(ParseError),
    /// Got an unexpected node.
    UnexpectedNode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Inconsistent(ref msg) => write!(f, "Inconsistent structure or data: {}", msg),
            Error::InvalidAttribute(ref name) => write!(f, "Invalid attribute for node: {}", name),
            Error::MissingNode(ref name) => write!(f, "Missing node: {}", name),
            Error::UnexpectedNode(ref name) => write!(f, "Unexpected node: {}", name),
            _ => write!(f, "{}", (self as &error::Error).description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Inconsistent(_) => "Inconsistent structure or data",
            Error::InvalidAttribute(_) => "Invalid node attribute",
            Error::MissingNode(_) => "Missing node",
            Error::UnexpectedNode(_) => "Unexpected node",
            Error::Parse(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Parse(ref err) => Some(err),
            _ => None,
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
