//! Wrapper for `std::io::Read`.

use std::fmt;
use std::io;
use byteorder::{ReadBytesExt, LittleEndian};


/// Reader with position info.
pub struct CountReader<R> {
    /// Source reader.
    source: R,
    /// Current position from the start of the stream..
    position: u64,
}

impl<R: io::Read> CountReader<R> {
    /// Creates a new `CountReader`.
    pub fn new(source: R) -> Self {
        CountReader {
            source: source,
            position: 0,
        }
    }

    /// Returns the current position from the start of the stream.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Skips to the given position.
    ///
    /// # Panics
    /// Panics if a byte at the given position has been already read.
    pub fn skip_to(&mut self, next_pos: u64) -> io::Result<()> {
        use std::io::Read;

        assert!(next_pos >= self.position(),
                "Destination position should be after current position");
        const TEMP_BUF_LEN: usize = 256;
        let mut temp_buf = [0u8; TEMP_BUF_LEN];
        let mut rest_len = next_pos - self.position();
        while rest_len > TEMP_BUF_LEN as u64 {
            try!(self.read_exact(&mut temp_buf));
            rest_len -= TEMP_BUF_LEN as u64;
        }
        try!(self.read_exact(&mut temp_buf[0..rest_len as usize]));

        assert_eq!(self.position(), next_pos);
        Ok(())
    }

    /// Reads and returns `u8` value.
    pub fn read_u8(&mut self) -> io::Result<u8> {
        ReadBytesExt::read_u8(self)
    }
}

macro_rules! impl_read_primitive {
    ($name:ident : $t:ty) => {
        /// Reads the value as little endian and returns it.
        pub fn $name(&mut self) -> io::Result<$t> {
            ReadBytesExt::$name::<LittleEndian>(self)
        }
    };
    ($($name:ident : $t:ty),*,) => {
        impl<R: io::Read> CountReader<R> {
            $(impl_read_primitive!($name: $t);)*
        }
    };
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

impl<R: io::Read> io::Read for CountReader<R> {
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

impl<R> fmt::Debug for CountReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CountReader")
            .field("position", &self.position)
            .finish()
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};
    use super::CountReader;

    fn do_test_skip_to(buf_size: usize, skip_dest: u64) {
        let mut short_buf = Cursor::new(vec![0; buf_size]);
        let short_count = {
            let mut reader = CountReader::new(&mut short_buf);
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
