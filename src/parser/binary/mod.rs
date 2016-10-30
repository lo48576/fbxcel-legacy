//! FBX binary parser.

use std::fmt;
use std::io::Read;

pub use self::error::{Result, Error, Warning};
pub use self::event::{Event, FbxHeader, FbxFooter, StartNode};
use self::reader::CountReader;

mod error;
mod event;
mod reader;


/// Parser state without error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Reading file header.
    Header,
    /// A node started.
    NodeStarted,
    /// A node ended.
    NodeEnded,
}


/// Pull parser for FBX binary format.
// I want to use `#[derive(Debug)]` but it causes compile error for rustc-1.12(stable), 1.13(beta),
// 1.14(nightly).
// See also: #[derive] is too conservative with field trait bounds · Issue #26925 · rust-lang/rust
//           ( https://github.com/rust-lang/rust/issues/26925 ).
pub struct BinaryParser<R> {
    /// Source reader.
    source: CountReader<R>,
    /// Parser state.
    ///
    /// `Ok(State)` if the parser is working without critical error,
    /// `Err(Error)` if the parsing failed and cannot be continued.
    state: Result<State>,
    /// Warnings.
    warnings: Vec<Warning>,
}

impl<R: Read> BinaryParser<R> {
    /// Creates a new binary parser.
    pub fn new(source: R) -> Self {
        BinaryParser {
            source: CountReader::new(source),
            state: Ok(State::Header),
            warnings: Vec::new(),
        }
    }

    /// Returns the parser error if available.
    pub fn error(&self) -> Option<&Error> {
        self.state.as_ref().err()
    }

    /// Returns reference to the warnings.
    pub fn warnings(&self) -> &Vec<Warning> {
        &self.warnings
    }

    /// Set the parser state as error.
    fn set_error(&mut self, err: &Error) {
        error!("FBX binary parser error: {}", err);
        debug!("Parser: {:#?}", self);
        self.state = Err(err.clone());
    }

    /// Add warning.
    fn warn(&mut self, warning: Warning) {
        warn!("FBX binary parser warning: {}", warning);
        debug!("Parser: {:#?}", self);
        self.warnings.push(warning);
    }
}

impl<R> fmt::Debug for BinaryParser<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BinaryParser")
            .field("source", &self.source)
            .field("state", &self.state)
            .field("warnings", &self.warnings)
            .finish()
    }
}
