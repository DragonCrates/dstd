use core::ffi::c_int;

extern crate alloc;
use alloc::string::String;

use crate::sys::libc::{c_size_t, errno};

pub type RawError = c_int;

pub fn last_os_error() -> RawError {
    unsafe { *errno::errno() }
}

unsafe extern "C" {
    // glibc
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[link_name = "__xpg_strerror_r"]
    fn strerror_r(errnum: c_int, buf: *mut u8, size: c_size_t) -> c_int;
    // musl, bionic
    #[cfg(any(target_os = "android", target_env = "musl"))]
    fn strerror_r(errnum: c_int, buf: *mut u8, size: c_size_t) -> c_int;
}

pub fn strerror(errno: RawError) -> String {
    let mut buf = [0_u8; 128];
    // Returns 0 on error, but buffer will still have contents after this call
    unsafe { strerror_r(errno, buf.as_mut_ptr(), buf.len()); }
    let zero = buf.iter().position(|&i| i == 0).expect("unterminated C string");
    String::from_utf8_lossy(&buf[..zero]).into()
}
