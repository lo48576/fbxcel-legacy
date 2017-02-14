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
    /// Attribute is invalid.
    ///
    /// This includes the cases below:
    /// - The node has too few or too many node attributes
    /// - The node has wrong types of node attribute values.
    /// - The node has wrong (or unsupported) values of node attributes values.
    InvalidAttribute(String),
    /// Required node is missing.
    MissingNode {
        /// Parent node.
        parent: String,
        /// The missing node, which is child node of the `parent` node.
        ///
        /// This may be `None` if the missing node is unknown or cannot be identified.
        child: Option<String>,
    },
    /// Parse error (including I/O error).
    Parse(ParseError),
    /// Got an unexpected node.
    UnexpectedNode(String),
}

impl Error {
    /// Creates a new `Error::MissingNode`.
    pub fn missing_node<'a, S: Into<String>, T: Into<Option<&'a str>>>(
        parent: S,
        child: T
    ) -> Self {
        Error::MissingNode {
            parent: parent.into(),
            child: child.into().map(|s| s.to_owned()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidAttribute(ref name) => write!(f, "Invalid attribute for node: {}", name),
            Error::MissingNode { ref parent, ref child } => {
                if let Some(child) = child.as_ref() {
                    write!(f, "Missing node: {} (parent={})", child, parent)
                } else {
                    write!(f, "Missing node: parent={}", parent)
                }
            },
            Error::UnexpectedNode(ref name) => write!(f, "Unexpected node: {}", name),
            _ => write!(f, "{}", (self as &error::Error).description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidAttribute(_) => "Invalid node attribute",
            Error::MissingNode { .. } => "Missing node",
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
