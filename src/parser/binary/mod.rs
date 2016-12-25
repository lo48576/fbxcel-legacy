//! FBX binary parser.

use std::io;
use std::io::Read;

pub use self::error::{Result, Error, Warning};
pub use self::event::{Event, FbxHeader, FbxFooter, StartNode};
pub use self::event::{Attributes, Attribute, SpecialAttributeType};
pub use self::event::{PrimitiveAttribute, ArrayAttribute, SpecialAttribute};
pub use self::event::ArrayAttributeReader;
use self::event::{EventBuilder, NodeHeader, StartNodeBuilder};
use self::event::read_fbx_header;
pub use self::reader::{ParserSource, BasicSource, SeekableSource, LimitedSeekReader};

mod error;
mod event;
mod reader;
pub mod utils;


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


/// Information about opened (but not yet closed) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OpenNode {
    /// Start offset of the node attribute.
    ///
    /// Note that this doesn't mean start offset of node header.
    begin: u64,
    /// End offset of the node.
    end: u64,
    /// End offset of attributes of the node.
    attributes_end: u64,
}


/// Pull parser for whole FBX with binary format.
#[derive(Debug)]
pub struct RootParser<R> {
    /// Source reader.
    source: R,
    /// Parser state.
    ///
    /// `Ok(State)` if the parser is working without critical error,
    /// `Err(Error)` if the parsing failed and cannot be continued.
    state: Result<State>,
    /// Warnings.
    warnings: Vec<Warning>,
    /// FBX version.
    fbx_version: Option<u32>,
    /// Open nodes stack.
    open_nodes: Vec<OpenNode>,
}

impl<R: Read> RootParser<BasicSource<R>> {
    /// Creates a new binary parser.
    pub fn new(source: R) -> Self {
        RootParser {
            source: BasicSource::new(source),
            state: Ok(State::Header),
            warnings: Vec::new(),
            fbx_version: None,
            open_nodes: Vec::new(),
        }
    }
}

impl<R: Read + io::Seek> RootParser<SeekableSource<R>> {
    /// Creates a new binary parser.
    pub fn from_seekable(source: R) -> Self {
        RootParser {
            source: SeekableSource::new(source),
            state: Ok(State::Header),
            warnings: Vec::new(),
            fbx_version: None,
            open_nodes: Vec::new(),
        }
    }
}

impl<R: ParserSource> RootParser<R> {
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
        let builder = match self.state.clone()? {
            State::Header => self.read_fbx_header(),
            State::NodeStarted => self.read_after_node_start(),
            State::NodeEnded => self.read_after_node_end(),
        };
        if let Err(ref err) = builder {
            self.set_error(err);
        }
        Ok(builder?.build(self))
    }

    /// Skips to the end of the current node.
    ///
    /// Returns `Ok(true)` if the current node is skipped and closed,
    /// `Ok(false)` if no nodes are open (i.e. the parser is reading under implicit root node),
    /// `Err(err)` if error happened.
    pub fn skip_current_node(&mut self) -> Result<bool> {
        if let Some(end) = self.open_nodes.pop().map(|v| v.end) {
            self.source.skip_to(end)?;
            self.state = Ok(State::NodeEnded);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns the number of the opened (and not closed) node.
    pub fn num_open_nodes(&self) -> usize {
        self.open_nodes.len()
    }

    /// Set the parser state as finished parsing.
    fn set_finish(&mut self) {
        self.state = Err(Error::Finished);
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
        let header = read_fbx_header(self)?;
        self.fbx_version = Some(header.version);
        self.state = Ok(State::NodeEnded);
        Ok(header.into())
    }

    /// Gets event after node start.
    fn read_after_node_start(&mut self) -> Result<EventBuilder> {
        // Attributes of recent opened node might remain partially unread.
        // They should be skipped before getting a next node event.
        self.skip_attributes()?;

        // Most recent opened node might ends here without a null node header.
        if let Some(end) = self.open_nodes.last().map(|v| v.end) {
            if self.source.position() == end {
                // Most recent opened node ends here (without a null node header).
                self.state = Ok(State::NodeEnded);
                self.open_nodes.pop();
                return Ok(EventBuilder::EndNode);
            }
        }

        let builder = self.read_node_event()?;
        Ok(builder)
    }

    /// Gets event after node end.
    fn read_after_node_end(&mut self) -> Result<EventBuilder> {
        self.read_node_event()
    }

    /// Reads a next node-related event from the source.
    ///
    /// This always returns `Ok(EventBuilder::StartNode)`, `Ok(EventBuilder::EndNode)`,
    /// `Ok(EventBuilder::EndFbx)` or `Err(_)`.
    fn read_node_event(&mut self) -> Result<EventBuilder> {
        let header = NodeHeader::read_from_parser(self)?;
        if header.is_node_end() {
            if let Some(last_node) = self.open_nodes.pop() {
                // There is open nodes, so this is not end of the FBX.
                let current_pos = self.source.position();
                if current_pos != last_node.end {
                    // Invalid node header.
                    return Err(Error::WrongNodeEndOffset {
                        begin: last_node.begin,
                        expected_end: last_node.end,
                        real_end: current_pos,
                    });
                }
            } else {
                assert_eq!(self.state.as_ref().ok(),
                           Some(&State::NodeEnded),
                           "End of implicit root node is read with unexpected parser state {:?}",
                           self.state);
                // No open nodes, so this `EndNode` event indicates the end of
                // the implicit root node.
                // FBX file has no more nodes.
                let footer = self.read_fbx_footer();
                return Ok(footer.into());
            }

            self.state = Ok(State::NodeEnded);
            Ok(EventBuilder::EndNode)
        } else {
            let mut name_vec = vec![0u8; header.bytelen_name as usize];
            self.source.read_exact(&mut name_vec)?;
            let name = String::from_utf8(name_vec).map_err(Error::node_name_invalid_utf8)?;
            let current_pos = self.source.position();
            self.open_nodes.push(OpenNode {
                begin: current_pos,
                end: header.end_offset,
                attributes_end: current_pos + header.bytelen_attributes,
            });

            // Zero or more attributes come after node start.
            self.state = Ok(State::NodeStarted);
            Ok(StartNodeBuilder {
                name: name,
                header: header,
            }
            .into())
        }
    }

    /// Reads an FBX footer.
    fn read_fbx_footer(&mut self) -> Result<FbxFooter> {
        self.set_finish();
        FbxFooter::read_from_parser(self)
    }

    /// Skip attributes of the most recent opened node.
    fn skip_attributes(&mut self) -> io::Result<()> {
        let attributes_end = self.open_nodes
            .last()
            .expect("`RootParser::skip_attributes()` is called but no nodes are open")
            .attributes_end;
        self.source.skip_to(attributes_end)
    }
}


/// Pull parser for a subtree of the FBX binary.
pub struct SubtreeParser<'a, R: 'a> {
    /// Root parser.
    root_parser: &'a mut RootParser<R>,
    /// Initial depth.
    ///
    /// Depth of the implicit root node is `0`.
    initial_depth: usize,
}

impl<'a, R: 'a + ParserSource> SubtreeParser<'a, R> {
    /// Creates a new `SubtreeParser`.
    pub fn new(root_parser: &'a mut RootParser<R>) -> Self {
        let initial_depth = root_parser.num_open_nodes();
        SubtreeParser {
            root_parser: root_parser,
            initial_depth: initial_depth,
        }
    }

    /// Checks if the subtree parser can emit more events.
    ///
    /// Returns `Ok(())` if more events can be read,
    /// `Err(Error::Finished)` if the subtree is all read,
    /// `Err(_)` if error happened.
    fn check_finished(&self) -> Result<()> {
        if let Some(err) = self.root_parser.error() {
            return Err(err.clone());
        }
        if self.root_parser.num_open_nodes() < self.initial_depth {
            return Err(Error::Finished);
        }
        Ok(())
    }

    /// Parses FBX from the given stream and returns the next event.
    pub fn next_event(&mut self) -> Result<Event<R>> {
        self.check_finished()?;
        self.root_parser.next_event()
    }

    /// Skips to the end of the current node.
    ///
    /// Returns `Ok(true)` if the current node is skipped and closed,
    /// `Ok(false)` if no nodes are open (i.e. the parser is reading under implicit root node),
    /// `Err(err)` if error happened.
    pub fn skip_current_node(&mut self) -> Result<bool> {
        self.check_finished()?;
        self.root_parser.skip_current_node()
    }
}
