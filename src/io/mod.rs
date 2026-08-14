extern crate alloc;
use alloc::vec::Vec;

#[cfg(windows)]
use crate::sys::windows::types::*;

pub(crate) mod stdio;
pub use stdio::{Stdin, stdin, Stdout, stdout, Stderr, stderr};
mod error;
pub use error::{Result, Error, RawError};

use error::Repr;

// TODO: move this to windows.rs
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

pub enum SeekFrom {
    Start(u64),
    Current(i64),
    End(i64),
}

impl SeekFrom {
    pub(crate) fn to_flags(&self) -> (i64, i32) {
        match *self {
            SeekFrom::Start(off) => (off as i64, 0),
            SeekFrom::Current(off) => (off, 1),
            SeekFrom::End(off) => (off, 2),
        }
    }
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_len(&mut self) -> Result<u64> {
        let pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(len)
    }
 
    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}
