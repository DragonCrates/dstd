extern crate alloc;
use alloc::vec::Vec;

use crate::io::{Result, ErrorKind, const_error};

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            let nr = self.read(buf)?;
            if nr == 0 { return Err(const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer")); }
            buf = &mut buf[nr..];
        }
        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        let mut init = buf.len();

        if buf.capacity() < 128 { buf.reserve(128 - buf.len()); }
        buf.resize(buf.capacity(), 0);

        loop {
            if init + buf.len()/8 >= buf.len() {
                buf.resize(buf.capacity()*2, 0);
            }

            match self.read(&mut buf[init..]) {
                Ok(nr) => {
                    if nr == 0 {
                        buf.truncate(init);
                        return Ok(())
                    }

                    init += nr;
                }

                Err(err) => {
                    buf.truncate(init);
                    return Err(err)
                }
            }
        }
    }
}
