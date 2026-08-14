//! Definitions for items that should be accessible across entire crate

#![allow(non_camel_case_types)]

use core::ffi::c_int;

pub type c_ssize_t = isize;
pub type c_size_t = usize;
pub type c_off_t = i64;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod errno {
    use core::ffi::c_int;

    // Linux errno
    pub const EINTR: i32 = 4;
    pub const EAGAIN: i32 = 11;

    unsafe extern "C" {
        /// Returns the address of the calling thread's `errno` storage.
        #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
        #[cfg_attr(target_os = "android", link_name = "__errno")]
        pub fn errno() -> *mut c_int;
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod fcntl {
    use core::ffi::c_int;

    // Linux fcntl
    pub const O_RDONLY: c_int = 0;
    pub const O_WRONLY: c_int = 1;
    pub const O_RDWR: c_int = 2;
    pub const O_CREAT: c_int = 0o100;
    pub const O_EXCL: c_int = 0o200;
    pub const O_TRUNC: c_int = 0o1000;
    pub const O_APPEND: c_int = 0o2000;
}

#[cfg(unix)]
unsafe extern "C" {
    /// Open and possibly create a file
    pub fn open(path: *const u8, flags: c_int, ...) -> c_int;
    /// Read from a file descriptor
    pub fn read(fd: c_int, buf: *mut u8, count: c_size_t) -> c_ssize_t;
    /// Write to a file descriptor
    pub fn write(fd: c_int, buf: *const u8, count: c_size_t) -> c_ssize_t;
    /// Reposition read/write file offset
    pub fn lseek(fd: c_int, offset: c_off_t, whence: c_int) -> c_off_t;
    /// Close a file descriptor
    pub fn close(fd: c_int) -> c_int;
}
