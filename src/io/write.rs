use core::fmt::{self, Arguments};

extern crate alloc;
use alloc::vec::Vec;

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

    fn write_fmt(&mut self, args: Arguments<'_>) -> Result<()> {
        default_write_fmt(self, args)
    }
}

// Taken as-is from libstd
fn default_write_fmt<W: Write + ?Sized>(this: &mut W, args: fmt::Arguments<'_>) -> Result<()> {
    // Create a shim which translates a `Write` to a `fmt::Write` and saves off
    // I/O errors, instead of discarding them.
    struct Adapter<'a, T: ?Sized + 'a> {
        inner: &'a mut T,
        error: Result<()>,
    }

    impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            match self.inner.write_all(s.as_bytes()) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.error = Err(e);
                    Err(fmt::Error)
                }
            }
        }
    }

    let mut output = Adapter { inner: this, error: Ok(()) };
    match fmt::write(&mut output, args) {
        Ok(()) => Ok(()),
        Err(..) => {
            // Check whether the error came from the underlying `Write`.
            if output.error.is_err() {
                output.error
            } else {
                panic!("a formatting trait implementation returned an error when the underlying stream did not");
            }
        }
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend(buf);
        Ok(())
    }    
}
