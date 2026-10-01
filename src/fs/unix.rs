use core::ffi::c_int;

use super::OpenOptions;
use crate::io::{self, Error, Read, Write, Seek, SeekFrom};
use crate::os_str::OsStr;
use crate::sys::libc::Fd;
use crate::sys::libc::fcntl::{self, open};
use crate::sys::libc::unistd::{read, write, lseek, close};

pub type RawHandle = Fd;

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

pub struct File {
    fd: Fd,
}

impl File {
    pub fn open(name: &OsStr, opts: &OpenOptions) -> io::Result<File> {
        let flags = opts_to_flags(opts);
        let fd = unsafe { open(name.as_ptr(), flags, 0o666) };
        if fd == -1 { return Err(Error::last_os_error()); }
        Ok(File { fd })
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { read(self.fd, buf.as_mut_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { write(self.fd, buf.as_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = pos.to_flags();
        let ret = unsafe { lseek(self.fd, offset, whence) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as u64)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { close(self.fd); }
    }
}
