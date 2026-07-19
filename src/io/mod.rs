use crate::ffi::*;
use crate::prelude::Vec;

pub(crate) mod stdio;
pub use stdio::{Stdin, stdin, Stdout, stdout, Stderr, stderr};
mod error;
pub use error::{Result, Error, ErrorOs};

#[cfg(not(windows))]
pub(crate) use error::errno;

use error::Repr;

#[cfg(unix)]
unsafe extern "C" {
    /// Read from a file descriptor
    pub(crate) fn read(fd: c_int, buf: *mut u8, count: c_size_t) -> c_ssize_t;
    /// Write to a file descriptor
    pub(crate) fn write(fd: c_int, buf: *const u8, count: c_size_t) -> c_ssize_t;
    /// Close a file descriptor
    pub(crate) fn close(fd: c_int) -> c_int;
}

#[cfg(windows)]
unsafe extern "C" {
    /// Writes data to the specified file or input/output (I/O) device.
    pub(crate) fn WriteFile(
        /* [in] */ hFile: HANDLE,
        /* [in] */ lpBuffer: LPCVOID,
        /* [in] */ nNumberOfBytesToWrite: DWORD,
        /* [out, optional] */ lpNumberOfBytesWritten: LPDWORD,
        /* [in, out, optional] */ lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    /// Closes an open object handle.
    pub(crate) fn CloseHandle(
        /* [in] */ hObject: HANDLE
    ) -> BOOL;
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            let nr = self.read(buf)?;
            if nr == 0 { return Err(Error { repr: Repr::UnexpectedEof }); }
            buf = &mut buf[nr..];
        }
        Ok(())
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        let mut init = buf.len();
        if buf.capacity() == 0 { buf.reserve(512); }
        buf.resize(buf.capacity(), 0);
        loop {
            if init == buf.len() {
                buf.resize(buf.capacity()*2, 0);
            }
            let nr = self.read(&mut buf[init..])?;
            if nr == 0 { break; }
            init += nr;
        }
        buf.truncate(init);
        Ok(())
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            let nw = self.write(buf)?;
            if nw == 0 { return Err(Error { repr: Repr::WriteZero }); }
            buf = &buf[nw..];
        }
        Ok(())
    }
}
