//! Wrapper for `std::io::Read`.

use std::fmt;
use std::mem;
use std::io;


macro_rules! impl_read_little_endian_integer {
    ($ty:ident, $name:ident, $size:expr) => {
/// Reads a little-endian value and returns it.
        fn $name(&mut self) -> io::Result<$ty> {
            assert_eq!($size, mem::size_of::<$ty>());

            let mut data: $ty = 0;
            let mut slice = unsafe {
                ::std::slice::from_raw_parts_mut(&mut data as *mut $ty as *mut u8, $size)
            };
            try!(self.read_exact(slice));
            Ok($ty::from_le(data))
        }
    }
}

/// The `ReadLittleEndian` trait allows for reading little-endian primitive type values from a
/// source.
pub trait ReadLittleEndian: io::Read {
    impl_read_little_endian_integer!(u8, read_u8, 1);
    impl_read_little_endian_integer!(u32, read_u32, 4);
    impl_read_little_endian_integer!(u64, read_u64, 8);
    impl_read_little_endian_integer!(i16, read_i16, 2);
    impl_read_little_endian_integer!(i32, read_i32, 4);
    impl_read_little_endian_integer!(i64, read_i64, 8);

    /// Reads a little-endian value and returns it.
    fn read_f32(&mut self) -> io::Result<f32> {
        let integer = try!(self.read_u32());
        let val = unsafe { mem::transmute(integer) };
        Ok(val)
    }

    /// Reads a little-endian value and returns it.
    fn read_f64(&mut self) -> io::Result<f64> {
        let integer = try!(self.read_u64());
        let val = unsafe { mem::transmute(integer) };
        Ok(val)
    }
}

impl<R: io::Read> ReadLittleEndian for R {}


/// Source stream for `BinaryParser`.
pub trait ParserSource: fmt::Debug + io::Read {
    /// Returns the current position from the start of the stream.
    fn position(&self) -> u64;

    /// Skips to the given position.
    ///
    /// # Panics
    /// Panics if a byte at the given position has been already read.
    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()>;
}


/// Reader with position info.
pub struct BasicSource<R> {
    /// Source reader.
    source: R,
    /// Current position from the start of the stream..
    position: u64,
}

impl<R: io::Read> BasicSource<R> {
    /// Creates a new `BasicSource`.
    pub fn new(source: R) -> Self {
        BasicSource {
            source: source,
            position: 0,
        }
    }
}

impl<R: io::Read> io::Read for BasicSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_len = try!(self.source.read(buf));
        self.position += read_len as u64;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        try!(self.source.read_exact(buf));
        self.position += buf.len() as u64;
        Ok(())
    }
}

impl<R> fmt::Debug for BasicSource<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BasicSource")
            .field("position", &self.position)
            .finish()
    }
}

impl<R: io::Read> ParserSource for BasicSource<R> {
    fn position(&self) -> u64 {
        self.position
    }

    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()> {
        use std::io::Read;

        assert!(dest_pos >= self.position(),
                "Destination position should be after current position: dest_pos={}, position={}",
                dest_pos,
                self.position());
        const TEMP_BUF_LEN: usize = 256;
        let mut temp_buf = [0u8; TEMP_BUF_LEN];
        let mut rest_len = dest_pos - self.position();
        while rest_len > TEMP_BUF_LEN as u64 {
            try!(self.read_exact(&mut temp_buf));
            rest_len -= TEMP_BUF_LEN as u64;
        }
        try!(self.read_exact(&mut temp_buf[0..rest_len as usize]));

        assert_eq!(self.position(), dest_pos);
        Ok(())
    }
}


/// Reader with position info and seek feature.
///
/// This wrapper doesn't manage offset, i.e. the start of the source stream should be the start of
/// the FBX data.
pub struct SeekableSource<R> {
    /// Source reader.
    source: R,
    /// Current position from the start of the stream..
    position: u64,
}

impl<R: io::Read + io::Seek> SeekableSource<R> {
    /// Creates a new `SeekableSource`.
    pub fn new(source: R) -> Self {
        SeekableSource {
            source: source,
            position: 0,
        }
    }
}

impl<R: io::Read> io::Read for SeekableSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_len = try!(self.source.read(buf));
        self.position += read_len as u64;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        try!(self.source.read_exact(buf));
        self.position += buf.len() as u64;
        Ok(())
    }
}

impl<R: io::Read + io::Seek> io::Seek for SeekableSource<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.position = try!(self.source.seek(pos));
        Ok(self.position)
    }
}

impl<R> fmt::Debug for SeekableSource<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SeekableSource")
            .field("position", &self.position)
            .finish()
    }
}

impl<R: io::Read + io::Seek> ParserSource for SeekableSource<R> {
    fn position(&self) -> u64 {
        self.position
    }

    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()> {
        use std::io::{Seek, SeekFrom};

        assert!(dest_pos >= self.position(),
                "Destination position should be after current position: dest_pos={}, position={}",
                dest_pos,
                self.position());
        try!(self.seek(SeekFrom::Start(dest_pos)));

        assert_eq!(self.position(), dest_pos);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};
    use super::BasicSource;

    fn do_test_skip_to(buf_size: usize, skip_dest: u64) {
        let mut short_buf = Cursor::new(vec![0; buf_size]);
        let short_count = {
            let mut reader = BasicSource::new(&mut short_buf);
            reader.skip_to(skip_dest).expect("Failed to skip");
            reader.position()
        };
        assert_eq!(short_count, skip_dest);
        assert_eq!(short_count,
                   short_buf.seek(SeekFrom::Current(0)).expect("Failed to seek"));
    }

    #[test]
    fn test_skip_to() {
        do_test_skip_to(0, 0);
        do_test_skip_to(15, 0);
        do_test_skip_to(30, 23);
        do_test_skip_to(512, 401);
        do_test_skip_to(64, 64);
    }
}
