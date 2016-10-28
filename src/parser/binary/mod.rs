//! FBX binary parser.

use std::fmt;
use std::io::Read;

use self::reader::CountReader;

mod reader;


/// Pull parser for FBX binary format.
// I want to use `#[derive(Debug)]` but it causes compile error for rustc-1.12(stable), 1.13(beta),
// 1.14(nightly).
// See also: #[derive] is too conservative with field trait bounds · Issue #26925 · rust-lang/rust
//           ( https://github.com/rust-lang/rust/issues/26925 ).
pub struct BinaryParser<R> {
    /// Source reader.
    source: CountReader<R>,
}

impl<R: Read> BinaryParser<R> {
    /// Creates a new binary parser.
    pub fn new(source: R) -> Self {
        BinaryParser {
            source: CountReader::new(source),
        }
    }
}

impl<R> fmt::Debug for BinaryParser<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BinaryParser")
            .field("source", &self.source)
            .finish()
    }
}
