use core::ffi::{c_int, c_void};

unsafe extern "C" {
    /// Read from a file descriptor
    pub fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    /// Write to a file descriptor
    pub fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    /// Reposition read/write file offset
    pub fn lseek(fd: c_int, offset: i64, whence: c_int) -> i64;
    /// Close a file descriptor
    pub fn close(fd: c_int) -> c_int;
    /// Exit terminates the current thread
    pub fn exit(code: i32) -> !;
}
