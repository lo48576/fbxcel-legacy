//! Array type node attribute.

use std::fmt;
use std::marker::PhantomData;
use std::io;
use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian};
#[cfg(feature = "flate2")]
use flate2::read::ZlibDecoder;
#[cfg(feature = "libflate")]
use libflate::zlib;

use parser::binary::{BinaryParser, CountReader};
use parser::binary::error::{Result, Error, Warning};


/// Read array type attribute from the given parser.
pub fn read_array_attribute<R: Read>(
    parser: &mut BinaryParser<R>,
    type_code: u8
) -> Result<(ArrayAttribute<R>, u64)> {
    let header = try!(ArrayAttributeHeader::read_from_parser(parser));
    let current_pos = parser.source.count();
    let BinaryParser { ref mut source, ref mut warnings, .. } = *parser;
    let reader = try!(ArrayDecoder::new(source, &header));

    let value = match type_code {
        b'b' => ArrayAttribute::Bool(ArrayAttributeReader::new(&header, reader, warnings)),
        b'i' => ArrayAttribute::I32(ArrayAttributeReader::new(&header, reader, warnings)),
        b'l' => ArrayAttribute::I64(ArrayAttributeReader::new(&header, reader, warnings)),
        b'f' => ArrayAttribute::F32(ArrayAttributeReader::new(&header, reader, warnings)),
        b'd' => ArrayAttribute::F64(ArrayAttributeReader::new(&header, reader, warnings)),
        _ => unreachable!(),
    };
    Ok((value, current_pos + header.bytelen_elements as u64))
}


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
#[derive(Debug)]
pub enum ArrayAttribute<'a, R: 'a> {
    /// Array of `bool`.
    Bool(ArrayAttributeReader<'a, R, bool>),
    /// Array of `i32`.
    I32(ArrayAttributeReader<'a, R, i32>),
    /// Array of `i64`.
    I64(ArrayAttributeReader<'a, R, i64>),
    /// Array of `f32`.
    F32(ArrayAttributeReader<'a, R, f32>),
    /// Array of `f64`.
    F64(ArrayAttributeReader<'a, R, f64>),
}


/// Reader of array attribute elements.
#[derive(Debug)]
pub struct ArrayAttributeReader<'a, R: 'a, T> {
    num_elements: u64,
    rest_elements: u64,
    reader: ArrayDecoder<'a, CountReader<R>>,
    warnings: &'a mut Vec<Warning>,
    _value_type: PhantomData<T>,
}

impl<'a, R: 'a + Read, T> ArrayAttributeReader<'a, R, T> {
    fn new<'b>(
        header: &'b ArrayAttributeHeader,
        reader: ArrayDecoder<'a, CountReader<R>>,
        warnings: &'a mut Vec<Warning>
    ) -> Self {
        ArrayAttributeReader {
            num_elements: header.num_elements as u64,
            rest_elements: header.num_elements as u64,
            reader: reader,
            warnings: warnings,
            _value_type: PhantomData,
        }
    }

    /// Returns number of elements.
    pub fn num_elements(&self) -> u64 {
        self.num_elements
    }

    /// Returns number of rest elements.
    pub fn rest_elements(&self) -> u64 {
        self.num_elements
    }
}

impl<'a, R: 'a + Read> Iterator for ArrayAttributeReader<'a, R, bool> {
    type Item = io::Result<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest_elements == 0 {
            return None;
        }
        self.rest_elements -= 1;
        let raw = match self.reader.read_u8() {
            Ok(val) => val,
            Err(err) => return Some(Err(err)),
        };
        let val = (raw & 1) == 1;
        Some(Ok(val))
    }
}

macro_rules! impl_attr_array_iter {
    ($ty:ty, $f:ident) => {
        impl<'a, R: 'a + Read> Iterator for ArrayAttributeReader<'a, R, $ty> {
            type Item = io::Result<$ty>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.rest_elements == 0 {
                    return None;
                }
                self.rest_elements -= 1;
                Some(self.reader.$f::<LittleEndian>())
            }
        }
    }
}

impl_attr_array_iter!(i32, read_i32);
impl_attr_array_iter!(i64, read_i64);
impl_attr_array_iter!(f32, read_f32);
impl_attr_array_iter!(f64, read_f64);


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
