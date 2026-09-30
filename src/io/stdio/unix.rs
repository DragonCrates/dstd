use core::ffi::c_int;

use crate::io::{Result, Error, Read, Write};
use crate::sys::libc;

pub struct RawStdio(c_int);

const STDIN: c_int = 0;
const STDOUT: c_int = 1;
const STDERR: c_int = 2;

pub fn raw_stdin() -> RawStdio {
    RawStdio(STDIN)
}

pub fn raw_stdout() -> RawStdio {
    RawStdio(STDOUT)
}

pub fn raw_stderr() -> RawStdio {
    RawStdio(STDERR)
}

impl Read for RawStdio {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe { libc::read(self.0, buf.as_mut_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }
}

impl Write for RawStdio {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { libc::write(self.0, buf.as_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }
}
