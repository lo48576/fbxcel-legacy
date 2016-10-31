//! FBX binary parser.

use std::fmt;
use std::io::Read;

pub use self::error::{Result, Error, Warning};
pub use self::event::{Event, FbxHeader, FbxFooter, StartNode};
use self::event::EventBuilder;
use self::event::read_fbx_header;
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
    ///
    /// This status means:
    ///
    /// - there might be some attributes remain unread, or
    /// - the most recent opened node might close without no null node header.
    NodeStarted,
    /// A node ended.
    ///
    /// This status means:
    ///
    /// - if the next event is `NodeEnd`, there must be a null node header, and
    /// - if the parser got an extra null header, it indicates end of implicit root node.
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
    /// FBX version.
    fbx_version: Option<u32>,
}

impl<R: Read> BinaryParser<R> {
    /// Creates a new binary parser.
    pub fn new(source: R) -> Self {
        BinaryParser {
            source: CountReader::new(source),
            state: Ok(State::Header),
            warnings: Vec::new(),
            fbx_version: None,
        }
    }

    /// Returns FBX version of the reading input.
    ///
    /// Returns `None` if unknown yet.
    pub fn fbx_version(&self) -> Option<u32> {
        self.fbx_version
    }

    /// Returns the parser error if available.
    pub fn error(&self) -> Option<&Error> {
        self.state.as_ref().err()
    }

    /// Returns reference to the warnings.
    pub fn warnings(&self) -> &Vec<Warning> {
        &self.warnings
    }

    /// Parses FBX from the given stream and returns the next event.
    pub fn next_event(&mut self) -> Result<Event<R>> {
        let builder = match try!(self.state.clone()) {
            State::Header => self.read_fbx_header(),
            State::NodeStarted => self.read_after_node_start(),
            State::NodeEnded => self.read_after_node_end(),
        };
        if let Err(ref err) = builder {
            self.set_error(err);
        }
        Ok(try!(builder).build(self))
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

    /// Reads FBX header.
    fn read_fbx_header(&mut self) -> Result<EventBuilder> {
        let header = try!(read_fbx_header(self));
        self.fbx_version = Some(header.version);
        self.state = Ok(State::NodeEnded);
        Ok(header.into())
    }

    /// Gets event after node start.
    fn read_after_node_start(&mut self) -> Result<EventBuilder> {
        unimplemented!()
    }

    /// Gets event after node end.
    fn read_after_node_end(&mut self) -> Result<EventBuilder> {
        unimplemented!()
    }
}

impl<R> fmt::Debug for BinaryParser<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BinaryParser")
            .field("source", &self.source)
            .field("state", &self.state)
            .field("warnings", &self.warnings)
            .field("fbx_version", &self.fbx_version)
            .finish()
    }
}
