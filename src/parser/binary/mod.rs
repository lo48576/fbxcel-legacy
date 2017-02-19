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


/// Warnings store.
#[derive(Default, Debug, Clone)]
pub struct Warnings(Vec<Warning>);

impl Warnings {
    /// Creates a new `Warnings`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a warning.
    pub fn warn(&mut self, warning: Warning) {
        warn!("FBX binary parser warning: {}", warning);
        self.0.push(warning);
    }

    /// Returns the inner vector.
    pub fn inner(self) -> Vec<Warning> {
        self.0
    }
}

impl ::std::ops::Deref for Warnings {
    type Target = [Warning];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Parser of a FBX binary node.
pub trait Parser<R: ParserSource> {
    /// Returns the root parser.
    fn root_parser(&self) -> &RootParser<R>;
    /// Parses FBX from the given stream and returns the next event.
    fn next_event(&mut self) -> Result<Event<R>>;
    /// Skips to the end of the current node.
    ///
    /// Returns `Ok(true)` if the current node is skipped and closed,
    /// `Ok(false)` if no nodes are open (i.e. the parser is reading under implicit root node),
    /// `Err(err)` if error happened.
    fn skip_current_node(&mut self) -> Result<bool>;
    /// Creates subtree parser for the current node.
    fn subtree_parser(&mut self) -> SubtreeParser<R>;
}


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
    warnings: Warnings,
    /// FBX version.
    fbx_version: Option<u32>,
    /// Open nodes stack.
    open_nodes: Vec<OpenNode>,
    /// Node name of the recent opened node.
    recent_node_name: Option<String>,
}

impl<R: Read> RootParser<BasicSource<R>> {
    /// Creates a new binary parser.
    pub fn new(source: R) -> Self {
        RootParser {
            source: BasicSource::new(source),
            state: Ok(State::Header),
            warnings: Warnings::new(),
            fbx_version: None,
            open_nodes: Vec::new(),
            recent_node_name: None,
        }
    }
}

impl<R: Read + io::Seek> RootParser<SeekableSource<R>> {
    /// Creates a new binary parser.
    pub fn from_seekable(source: R) -> Self {
        RootParser {
            source: SeekableSource::new(source),
            state: Ok(State::Header),
            warnings: Warnings::new(),
            fbx_version: None,
            open_nodes: Vec::new(),
            recent_node_name: None,
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
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }

    /// Returns the number of the opened (and not closed) node.
    pub fn num_open_nodes(&self) -> usize {
        self.open_nodes.len()
    }

    /// Returns the node name of the recent opened node.
    pub fn recent_node_name(&self) -> Option<&str> {
        self.recent_node_name.as_ref().map(String::as_str)
    }

    /// Returns the node name of the recent opened node with ownership.
    pub fn take_recent_node_name(&mut self) -> Option<String> {
        self.recent_node_name.take()
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
        self.warnings.warn(warning);
        debug!("Parser: {:#?}", self);
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
            // Reuse node name buffer.
            self.recent_node_name = {
                // Take node name buffer inside the string if the buffer remains.
                // Create a new buffer if the buffer was already taken.
                let mut vecbuf = self.recent_node_name
                    .take()
                    .map(|s| {
                        // Get inner `Vec` of the string.
                        let mut v = s.into_bytes();
                        // Resize buffer.
                        // This reallocates only if the buffer is too small.
                        v.resize(header.bytelen_name as usize, 0);
                        v
                    })
                    .unwrap_or_else(|| vec![0; header.bytelen_name as usize]);
                // Read the node name into the buffer.
                self.source.read_exact(&mut vecbuf)?;
                // Covert the name into `String`.
                // If conversion failed, the buffer will be left empty.
                // This is ok because no more node events would be loaded and
                // the buffer would no longer be used.
                Some(String::from_utf8(vecbuf).map_err(Error::node_name_invalid_utf8)?)
            };

            let current_pos = self.source.position();
            self.open_nodes.push(OpenNode {
                begin: current_pos,
                end: header.end_offset,
                attributes_end: current_pos + header.bytelen_attributes,
            });

            // Zero or more attributes come after node start.
            self.state = Ok(State::NodeStarted);
            Ok(StartNodeBuilder { header: header }.into())
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

impl<R: ParserSource> Parser<R> for RootParser<R> {
    fn root_parser(&self) -> &RootParser<R> {
        self
    }

    fn next_event(&mut self) -> Result<Event<R>> {
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

    fn skip_current_node(&mut self) -> Result<bool> {
        if let Some(end) = self.open_nodes.pop().map(|v| v.end) {
            self.source.skip_to(end)?;
            self.state = Ok(State::NodeEnded);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn subtree_parser(&mut self) -> SubtreeParser<R> {
        SubtreeParser::new(self)
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
    fn ensure_not_finished(&self) -> Result<()> {
        if self.is_finished()? {
            Err(Error::Finished)
        } else {
            Ok(())
        }
    }

    /// Checks if the subtree parser finished reading events.
    fn is_finished(&self) -> Result<bool> {
        if let Some(err) = self.root_parser.error() {
            return Err(err.clone());
        }
        Ok(self.root_parser.num_open_nodes() < self.initial_depth)
    }

    /// Skip to the end of the subtree parser's readable range.
    ///
    /// Note that this is not intended to called for `SubtreeParser` reading implicit root node.
    /// If the subtree parser can read whole data, the parser skips all events including `FbxEnd`
    /// and returns `Err(Error::Finished)`.
    pub fn skip_to_end(self) -> Result<()> {
        if self.is_finished()? {
            return Ok(());
        }
        self.root_parser.open_nodes.truncate(self.initial_depth);
        if let Some(end) = self.root_parser.open_nodes.pop().map(|v| v.end) {
            self.root_parser.source.skip_to(end)?;
            self.root_parser.state = Ok(State::NodeEnded);
            Ok(())
        } else {
            self.root_parser.state = Err(Error::Finished);
            Err(Error::Finished)
        }
    }
}

impl<'a, R: 'a + ParserSource> Parser<R> for SubtreeParser<'a, R> {
    fn root_parser(&self) -> &RootParser<R> {
        self.root_parser
    }

    fn next_event(&mut self) -> Result<Event<R>> {
        self.ensure_not_finished()?;
        self.root_parser.next_event()
    }

    fn skip_current_node(&mut self) -> Result<bool> {
        self.ensure_not_finished()?;
        self.root_parser.skip_current_node()
    }

    fn subtree_parser(&mut self) -> SubtreeParser<R> {
        SubtreeParser::new(self.root_parser)
    }
}

impl<'a, R: ParserSource, P: Parser<R>> Parser<R> for &'a mut P {
    fn root_parser(&self) -> &RootParser<R> {
        (**self).root_parser()
    }

    fn next_event(&mut self) -> Result<Event<R>> {
        (**self).next_event()
    }

    fn skip_current_node(&mut self) -> Result<bool> {
        (**self).skip_current_node()
    }

    fn subtree_parser(&mut self) -> SubtreeParser<R> {
        (**self).subtree_parser()
    }
}
