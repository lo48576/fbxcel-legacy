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
    /// I/O error.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", (self as &error::Error).description())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Finished => "Successfully finished parsing and there are no more data",
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
// FIXME: This should be enum.
pub struct Warning {}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl error::Error for Warning {
    fn description(&self) -> &str {
        unimplemented!()
    }
}
