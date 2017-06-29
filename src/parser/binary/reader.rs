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
            self.read_exact(slice)?;
            Ok($ty::from_le(data))
        }
    }
}

macro_rules! impl_read_little_endian_integer_array {
    ($ty:ident, $name:ident, $size:expr) => {
/// Reads a little-endian values into the given buffer.
        fn $name(&mut self, buf: &mut [$ty]) -> io::Result<()> {
            assert_eq!($size, mem::size_of::<$ty>());

            {
                let mut slice = unsafe {
                    ::std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, $size * buf.len())
                };
                self.read_exact(slice)?;
            }
            for elem in buf {
                *elem = $ty::from_le(*elem);
            }
            Ok(())
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
        let integer = self.read_u32()?;
        let val = unsafe { mem::transmute(integer) };
        Ok(val)
    }

    /// Reads a little-endian value and returns it.
    fn read_f64(&mut self) -> io::Result<f64> {
        let integer = self.read_u64()?;
        let val = unsafe { mem::transmute(integer) };
        Ok(val)
    }

    impl_read_little_endian_integer_array!(u8, read_u8_arr, 1);
    impl_read_little_endian_integer_array!(u32, read_u32_arr, 4);
    impl_read_little_endian_integer_array!(u64, read_u64_arr, 8);
    impl_read_little_endian_integer_array!(i16, read_i16_arr, 2);
    impl_read_little_endian_integer_array!(i32, read_i32_arr, 4);
    impl_read_little_endian_integer_array!(i64, read_i64_arr, 8);

    /// Reads a little-endian values into the given buffer.
    fn read_f32_arr(&mut self, buf: &mut [f32]) -> io::Result<()> {
        self.read_u32_arr(unsafe { mem::transmute(buf) })
    }

    /// Reads a little-endian values into the given buffer.
    fn read_f64_arr(&mut self, buf: &mut [f64]) -> io::Result<()> {
        self.read_u64_arr(unsafe { mem::transmute(buf) })
    }
}

impl<R: io::Read> ReadLittleEndian for R {}


/// Source stream for `RootParser`.
pub trait ParserSource: fmt::Debug + io::Read {
    /// Returns the current position from the start of the stream.
    fn position(&self) -> u64;

    /// Skips to the given position.
    ///
    /// # Panics
    /// Panics if a byte at the given position has been already read.
    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()>;
}

impl<'a, R: ParserSource> ParserSource for &'a mut R {
    fn position(&self) -> u64 {
        (**self).position()
    }

    fn skip_to(&mut self, dest_pos: u64) -> io::Result<()> {
        (**self).skip_to(dest_pos)
    }
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
        let read_len = self.source.read(buf)?;
        self.position += read_len as u64;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.source.read_exact(buf)?;
        self.position += buf.len() as u64;
        Ok(())
    }
}

impl<R: io::BufRead> io::BufRead for BasicSource<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.position += amt as u64;
        self.source.consume(amt);
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
            self.read_exact(&mut temp_buf)?;
            rest_len -= TEMP_BUF_LEN as u64;
        }
        self.read_exact(&mut temp_buf[0..rest_len as usize])?;

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
        let read_len = self.source.read(buf)?;
        self.position += read_len as u64;
        Ok(read_len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.source.read_exact(buf)?;
        self.position += buf.len() as u64;
        Ok(())
    }
}

impl<R: io::Read + io::Seek> io::Seek for SeekableSource<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.position = self.source.seek(pos)?;
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
        self.seek(SeekFrom::Start(dest_pos))?;

        assert_eq!(self.position(), dest_pos);
        Ok(())
    }
}

impl<R: io::BufRead> io::BufRead for SeekableSource<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.position += amt as u64;
        self.source.consume(amt);
    }
}


/// Reader which can read and seek limited area of a stream.
pub struct LimitedSeekReader<R> {
    /// Source stream.
    source: R,
    /// Current position
    current: u64,
    /// Start position.
    begin: u64,
    /// End position.
    end: u64,
}

impl<R: io::Read> LimitedSeekReader<R> {
    /// Creates a new `LimitedSeekReader`.
    pub fn new(source: R, current: u64, begin: u64, end: u64) -> Self {
        assert!(begin <= end);
        assert!(begin <= current);
        assert!(current <= end);

        LimitedSeekReader {
            source: source,
            current: current,
            begin: begin,
            end: end,
        }
    }
}

impl<R> LimitedSeekReader<R> {
    /// Returns length of region allowed to be read in bytes.
    pub fn len(&self) -> u64 {
        assert!(self.begin <= self.end);
        self.end - self.begin
    }

    /// Returns `true` if the reader can read no data.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns rest length of region allowed to be read in bytes.
    fn rest_len(&self) -> u64 {
        assert!(self.current <= self.end);
        self.end - self.current
    }

    /// Returns the current position relative to the start of the limited region.
    fn rel_pos(&self) -> u64 {
        assert!(self.begin <= self.current);
        self.current - self.begin
    }
}

impl<R: io::Read> io::Read for LimitedSeekReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let limit = self.rest_len();
        let size = self.source.by_ref().take(limit).read(buf)?;
        self.current += size as u64;
        Ok(size)
    }
}

impl<R: io::BufRead> io::BufRead for LimitedSeekReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.current += amt as u64;
        self.source.consume(amt);
    }
}

impl<R: io::Seek> io::Seek for LimitedSeekReader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(val) => {
                let offset = ::std::cmp::min(self.len(), val);
                let target = self.begin + offset;
                assert!(target >= self.begin);
                assert!(target <= self.end);
                self.current = self.source.seek(io::SeekFrom::Start(target))?;
            },
            io::SeekFrom::Current(val) => {
                let offset = if val < 0 {
                    ::std::cmp::max(-(self.rel_pos() as i64), val)
                } else {
                    ::std::cmp::min(self.rest_len(), val as u64) as i64
                };
                let target = self.current as i64 + offset;
                assert!(target as u64 >= self.begin);
                assert!(target as u64 <= self.end);
                self.current = self.source.seek(io::SeekFrom::Current(offset))?;
            },
            io::SeekFrom::End(val) => {
                let offset = ::std::cmp::max(-(self.len() as i64), ::std::cmp::min(0, val));
                let target = (self.end as i64 + offset) as u64;
                assert!(target >= self.begin);
                assert!(target <= self.end);
                self.current = self.source.seek(io::SeekFrom::Start(target))?;
            },
        }
        Ok(self.rel_pos())
    }
}

impl<R> fmt::Debug for LimitedSeekReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LimitedSeekReader")
            .field("current", &self.current)
            .field("begin", &self.begin)
            .field("end", &self.end)
            .finish()
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};
    use super::{ParserSource, BasicSource, SeekableSource};

    fn do_test_skip_to(buf_size: usize, skip_dest: u64) {
        do_test_basic_skip_to(buf_size, skip_dest);
        do_test_seekable_skip_to(buf_size, skip_dest);
    }

    fn do_test_basic_skip_to(buf_size: usize, skip_dest: u64) {
        let mut short_buf = Cursor::new(vec![0; buf_size]);
        let short_count = {
            let mut reader = BasicSource::new(&mut short_buf);
            reader.skip_to(skip_dest).expect("Failed to skip");
            reader.position()
        };
        assert_eq!(short_count, skip_dest);
        assert_eq!(short_count,
                   short_buf
                       .seek(SeekFrom::Current(0))
                       .expect("Failed to seek"));
    }

    fn do_test_seekable_skip_to(buf_size: usize, skip_dest: u64) {
        let mut short_buf = Cursor::new(vec![0; buf_size]);
        let mut reader = SeekableSource::new(&mut short_buf);
        reader.skip_to(skip_dest).expect("Failed to skip");
        assert_eq!(reader.position(), skip_dest);
        reader.seek(SeekFrom::End(0)).expect("Failed to seek");;
        assert_eq!(reader.position(), buf_size as u64);
        reader.seek(SeekFrom::Start(0)).expect("Failed to seek");
        assert_eq!(reader.position(), 0);
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
