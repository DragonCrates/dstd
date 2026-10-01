use core::ffi::c_int;
use super::{c_size_t, c_ssize_t, c_off_t};

unsafe extern "C" {
    /// Read from a file descriptor
    pub fn read(fd: c_int, buf: *mut u8, count: c_size_t) -> c_ssize_t;
    /// Write to a file descriptor
    pub fn write(fd: c_int, buf: *const u8, count: c_size_t) -> c_ssize_t;
    /// Reposition read/write file offset
    pub fn lseek(fd: c_int, offset: c_off_t, whence: c_int) -> c_off_t;
    /// Close a file descriptor
    pub fn close(fd: c_int) -> c_int;
}
