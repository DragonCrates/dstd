use core::mem;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::io::{Result, Read};
use crate::io::buf::{BufTrait, Buf, Stack, Heap};

fn read_until_default<T>(this: &mut T, byte: u8, buf: &mut Vec<u8>) -> Result<usize>
where
    T: BufRead + ?Sized
{
    let mut nw = 0;
    loop {
        let this_buf = this.fill_buf()?;

        if this_buf.is_empty() {
            return Ok(nw);
        }

        // Try to find byte
        if let Some(pos) = this_buf.iter().position(|&i| i == byte) {
            buf.extend(&this_buf[..pos+1]);
            nw += pos+1;
            this.consume(pos+1);
            return Ok(nw);
        } else {
            buf.extend(this_buf);
            let len = this_buf.len();
            nw += len;
            this.consume(len);
        }
    }
}

pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amount: usize);

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        read_until_default(self, byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let safe_utf8 = buf.len();
        let mut bytes_vec = mem::take(buf).into_bytes();

        let ret = read_until_default(self, b'\n', &mut bytes_vec);

        match str::from_utf8(&bytes_vec[safe_utf8..]) {
            Ok(_) => {
                // SAFETY: we just verified that newly read bytes are valid unicode
                *buf = unsafe { String::from_utf8_unchecked(bytes_vec) };
                ret
            }

            Err(err) => {
                bytes_vec.truncate(safe_utf8);
                // SAFETY: we just discarded invalid bytes, restoring previous known-valid state
                *buf = unsafe { String::from_utf8_unchecked(bytes_vec) };
                Err(err.into())
            }
        }
    }
}

pub struct BufReader<T>(BufReaderInner<Heap, T>);

impl<T> BufReader<T> {
    pub fn new(io: T) -> BufReader<T> {
        let inner = BufReaderInner::new(io);
        BufReader(inner)
    }
}

impl<T: Read> Read for BufReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl<T: Read> BufRead for BufReader<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.0.consume(amount);
    }
}

pub struct BufReaderInner<B, T> {
    buf: Buf<B>,
    io: T,
}

impl<B: BufTrait, T> BufReaderInner<B, T> {
    pub fn new(io: T) -> BufReaderInner<B, T> {
        BufReaderInner {
            buf: Buf::new(),
            io,
        }
    }
}

impl<T> BufReaderInner<Stack, T> {
    pub const fn const_new(io: T) -> BufReaderInner<Stack, T> {
        BufReaderInner {
            buf: Buf::const_new(),
            io,
        }
    }
}

impl<B: BufTrait, T: Read> Read for BufReaderInner<B, T> {
    #[allow(clippy::len_zero)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut nw = 0;

        // Empty the buffer, if it had something
        if self.buf.len() > 0 {
            if buf.len() <= self.buf.len() {
                // buf is smaller than internal buffer
                buf.copy_from_slice(&self.buf[..buf.len()]);
                self.buf.consume(buf.len());
                return Ok(buf.len());
            } else {
                // empty internal buffer
                nw = self.buf.len();
                buf[..nw].copy_from_slice(&self.buf);
                self.buf.consume(nw);
            }
        }

        // If the `len - nw` is more than half of our buffer, just read directly
        if buf.len() - nw > self.buf.capacity() / 2 {
            nw += self.io.read(&mut buf[nw..])?;
            return Ok(nw);
        }

        // Otherwise, fill the buffer again and copy from it
        self.fill_buf()?;

        if self.buf.is_empty() {
            return Ok(nw);
        }

        let buf_space = buf.len() - nw;
        let to_write = buf_space.min(self.buf.len());
        buf[nw..nw+to_write].copy_from_slice(&self.buf[..to_write]);
        self.buf.consume(to_write);
        Ok(nw + to_write)
    }
}

impl<B: BufTrait, T: Read> BufRead for BufReaderInner<B, T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.buf.is_empty() {
            let nr = self.io.read(self.buf.get_fill())?;
            self.buf.extend(nr);
        }

        Ok(&*self.buf)
    }

    fn consume(&mut self, amount: usize) {
        self.buf.consume(amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reader that panics if `read` is called after `should_not_read` is set
    struct GuardedReader {
        data: Vec<u8>,
        pos: usize,
        should_not_read: bool,
    }

    impl Read for GuardedReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            assert!(!self.should_not_read, "read the underlying source despite buffered data");
            let avail = &self.data[self.pos..];
            let n = avail.len().min(buf.len());
            buf[..n].copy_from_slice(&avail[..n]);
            self.pos += n;
            Ok(n)
        }
    }

    /// fill_buf must return buffered data without touching the underlying
    /// reader, otherwise it would block waiting for more input
    #[test]
    fn fill_buf_does_not_read_with_buffered_data() {
        let mut br = BufReader::new(GuardedReader { data: b"abc\ndef\n".to_vec(), pos: 0, should_not_read: false });

        assert_eq!(br.fill_buf().unwrap(), b"abc\ndef\n");

        br.0.io.should_not_read = true;
        br.consume(4);

        assert_eq!(br.fill_buf().unwrap(), b"def\n");
    }
}
