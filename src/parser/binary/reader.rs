//! Wrapper for `std::io::Read`.

use std::fmt;
use std::io;
use byteorder::{ReadBytesExt, LittleEndian};


/// Source stream for `BinaryParser`.
pub trait ParserSource: fmt::Debug + io::Read {
    /// Returns the current position from the start of the stream.
    fn position(&self) -> u64;

    /// Skips to the given position.
    ///
    /// # Panics
    /// Panics if a byte at the given position has been already read.
    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()>;

    /// Reads the little-endian value and returns it.
    fn read_u8(&mut self) -> io::Result<u8>;
    /// Reads the little-endian value and returns it.
    fn read_u32(&mut self) -> io::Result<u32>;
    /// Reads the little-endian value and returns it.
    fn read_u64(&mut self) -> io::Result<u64>;
    /// Reads the little-endian value and returns it.
    fn read_i16(&mut self) -> io::Result<i16>;
    /// Reads the little-endian value and returns it.
    fn read_i32(&mut self) -> io::Result<i32>;
    /// Reads the little-endian value and returns it.
    fn read_i64(&mut self) -> io::Result<i64>;
    /// Reads the little-endian value and returns it.
    fn read_f32(&mut self) -> io::Result<f32>;
    /// Reads the little-endian value and returns it.
    fn read_f64(&mut self) -> io::Result<f64>;
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

macro_rules! impl_read_primitive {
    ($name:ident : $t:ty) => {
        /// Reads the value as little endian and returns it.
        fn $name(&mut self) -> io::Result<$t> {
            ReadBytesExt::$name::<LittleEndian>(self)
        }
    };
    ($($name:ident : $t:ty),*,) => {
        $(impl_read_primitive!($name: $t);)*
    };
}

impl<R: io::Read> ParserSource for BasicSource<R> {
    fn position(&self) -> u64 {
        self.position
    }

    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()> {
        use std::io::Read;

        assert!(dest_pos >= self.position(),
                "Destination position should be after current position");
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

    fn read_u8(&mut self) -> io::Result<u8> {
        ReadBytesExt::read_u8(self)
    }

    impl_read_primitive!(
        read_i16: i16,
        read_i32: i32,
        read_u32: u32,
        read_i64: i64,
        read_u64: u64,
        read_f32: f32,
        read_f64: f64,
    );
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
