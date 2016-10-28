//! Wrapper for `std::io::Read`.

use std::fmt;
use std::io;


/// Reader with read bytes count.
pub struct CountReader<R> {
    /// Source reader.
    source: R,
    /// Read bytes count.
    count: u64,
}

impl<R: io::Read> CountReader<R> {
    /// Creates a new `CountReader`.
    pub fn new(source: R) -> Self {
        CountReader {
            source: source,
            count: 0,
        }
    }

    /// Returns current count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Skips to the given position.
    ///
    /// # Panics
    /// Panics if a byte at the given position has been already read.
    pub fn skip_to(&mut self, next_pos: u64) -> io::Result<()> {
        use std::io::Read;

        assert!(next_pos >= self.count(),
                "Destination position should be after current position");
        const TEMP_BUF_LEN: usize = 256;
        let mut temp_buf = [0u8; TEMP_BUF_LEN];
        let mut rest_len = next_pos - self.count();
        while rest_len > TEMP_BUF_LEN as u64 {
            try!(self.read_exact(&mut temp_buf));
            rest_len -= TEMP_BUF_LEN as u64;
        }
        try!(self.read_exact(&mut temp_buf[0..rest_len as usize]));

        assert_eq!(self.count(), next_pos);
        Ok(())
    }
}

impl<R: io::Read> io::Read for CountReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_len = try!(self.source.read(buf));
        self.count += read_len as u64;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        try!(self.source.read_exact(buf));
        self.count += buf.len() as u64;
        Ok(())
    }
}

impl<R> fmt::Debug for CountReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CountReader")
            .field("count", &self.count)
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
            reader.skip_to(skip_dest).unwrap();
            reader.count()
        };
        assert_eq!(short_count, skip_dest);
        assert_eq!(short_count, short_buf.seek(SeekFrom::Current(0)).unwrap());
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
