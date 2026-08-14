use core::ffi::c_int;

use super::OpenOptions;
use crate::io::{Result, Error, SeekFrom};
use crate::os_str::OsStr;
use crate::sys::libc::{self, fcntl};

pub type Handle = c_int;

fn opts_to_flags(opts: &OpenOptions) -> c_int {
    let mut flags = 0;
    match (opts.read, opts.write) {
        (false, false) => flags |= fcntl::O_RDONLY,
        (true, false) => flags |= fcntl::O_RDONLY,
        (false, true) => flags |= fcntl::O_WRONLY,
        (true, true) => flags |= fcntl::O_RDWR,
    }
    if opts.append { flags |= fcntl::O_APPEND; }
    if opts.truncate { flags |= fcntl::O_TRUNC; }
    if opts.create { flags |= fcntl::O_CREAT; }
    if opts.excl { flags |= fcntl::O_EXCL; }
    flags
}

pub fn open(name: &OsStr, opts: &OpenOptions) -> Result<c_int> {
    let flags = opts_to_flags(opts);
    let ret = unsafe { libc::open(name.as_ptr(), flags) };
    if ret == -1 { return Err(Error::last_os_error()); }
    Ok(ret)
}

pub fn read(fd: c_int, buf: &mut [u8]) -> Result<usize> {
    let ret = unsafe { libc::read(fd, buf.as_mut_ptr(), buf.len()) };
    if ret == -1 { return Err(Error::last_os_error()); }
    Ok(ret as usize)
}

pub fn write(fd: c_int, buf: &[u8]) -> Result<usize> {
    let ret = unsafe { libc::write(fd, buf.as_ptr(), buf.len()) };
    if ret == -1 { return Err(Error::last_os_error()); }
    Ok(ret as usize)
}

pub fn seek(fd: c_int, pos: SeekFrom) -> Result<u64> {
    let (offset, whence) = pos.to_flags();
    let ret = unsafe { libc::lseek(fd, offset, whence) };
    if ret == -1 { return Err(Error::last_os_error()); }
    Ok(ret as u64)
}
