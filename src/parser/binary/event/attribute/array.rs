//! Array type node attribute.

use std::fmt;
use std::io;
use std::io::Read;
#[cfg(feature = "flate2")]
use flate2::read::ZlibDecoder;
#[cfg(feature = "libflate")]
use libflate::zlib;

use parser::binary::BinaryParser;
use parser::binary::error::{Result, Error};


/// Header of an array attribute.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ArrayAttributeHeader {
    /// Number of elements.
    num_elements: u32,
    /// Array encoding, i.e. compression method.
    encoding: u32,
    /// Length of the array (excluding this header) in bytes.
    bytelen_elements: u32,
}

impl ArrayAttributeHeader {
    fn read_from_parser<R: Read>(parser: &mut BinaryParser<R>) -> io::Result<Self> {
        let num_elements = try!(parser.source.read_u32());
        let encoding = try!(parser.source.read_u32());
        let bytelen_elements = try!(parser.source.read_u32());

        Ok(ArrayAttributeHeader {
            num_elements: num_elements,
            encoding: encoding,
            bytelen_elements: bytelen_elements,
        })
    }
}


/// Array type attribute.
// FIXME: unimplemented.
#[derive(Debug)]
pub struct ArrayAttribute<'a, R: 'a>(&'a mut R);


/// Attribute array decoder.
enum ArrayDecoder<'a, R: 'a> {
    /// Non-compressed stream.
    ///
    /// `encoding` == 0.
    Raw(io::Take<&'a mut R>),
    /// Zlib-compressed stream.
    ///
    /// `encoding` == 1.
    #[cfg(feature = "flate2")]
    Zlib(ZlibDecoder<io::Take<&'a mut R>>),
    /// Zlib-compressed stream.
    ///
    /// `encoding` == 1.
    #[cfg(feature = "libflate")]
    Zlib(zlib::Decoder<io::Take<&'a mut R>>),
}

impl<'a, R: 'a + Read> ArrayDecoder<'a, R> {
    fn new(reader: &'a mut R, header: &ArrayAttributeHeader) -> Result<Self> {
        match header.encoding {
            0 => Ok(ArrayDecoder::Raw(reader.take(header.bytelen_elements as u64))),
            #[cfg(feature = "flate2")]
            1 => Ok(ArrayDecoder::Zlib(ZlibDecoder::new(reader.take(header.bytelen_elements as u64)))),
            #[cfg(feature = "libflate")]
            1 => Ok(ArrayDecoder::Zlib(try!(zlib::Decoder::new(reader.take(header.bytelen_elements as u64))))),
            _ => Err(Error::UnknownArrayAttributeEncoding(header.encoding)),
        }
    }
}

impl<'a, R: 'a + Read> Read for ArrayDecoder<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            ArrayDecoder::Raw(ref mut reader) => reader.read(buf),
            #[cfg(feature = "flate2")]
            ArrayDecoder::Zlib(ref mut reader) => reader.read(buf),
            #[cfg(feature = "libflate")]
            ArrayDecoder::Zlib(ref mut reader) => reader.read(buf),
        }
    }
}

impl<'a, R: 'a> fmt::Debug for ArrayDecoder<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "ArrayDecoder::{}",
               match *self {
                   ArrayDecoder::Raw(_) => "Raw",
                   #[cfg(feature = "flate2")]
                   ArrayDecoder::Zlib(_) => "Zlib",
                   #[cfg(feature = "libflate")]
                   ArrayDecoder::Zlib(_) => "Zlib",
               })
    }
}
