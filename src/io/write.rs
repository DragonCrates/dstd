use crate::io::{Result, ErrorKind, const_error};

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            let nw = self.write(buf)?;
            if nw == 0 { return Err(const_error!(ErrorKind::WriteZero, "failed to write whole buffer")); }
            buf = &buf[nw..];
        }
        Ok(())
    }
}
