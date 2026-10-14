use core::ffi::c_int;

extern crate alloc;
use alloc::string::String;

use crate::sys::libc::errno;
use crate::os_str::OsStr;

use super::ErrorKind;

pub type RawError = c_int;

pub fn last_os_error() -> RawError {
    unsafe { *errno::errno() }
}

mod libc {
    use core::ffi::c_int;

    unsafe extern "C" {
        // glibc
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        #[link_name = "__xpg_strerror_r"]
        pub fn strerror_r(errnum: c_int, buf: *mut u8, size: usize) -> c_int;
        // musl, bionic
        #[cfg(any(target_os = "android", target_env = "musl"))]
        pub fn strerror_r(errnum: c_int, buf: *mut u8, size: usize) -> c_int;

        pub fn gai_strerror(errcode: c_int) -> *const u8;
    }
}

pub fn strerror(errno: RawError) -> String {
    let mut buf = [0_u8; 128];
    // Returns 0 on error, but buffer will still have contents after this call
    unsafe { libc::strerror_r(errno, buf.as_mut_ptr(), buf.len()); }
    let zero = buf.iter().position(|&i| i == 0).expect("unterminated C string");
    String::from_utf8_lossy(&buf[..zero]).into()
}

#[cfg_attr(any(target_os = "linux", target_os = "android"), allow(unreachable_patterns))]
pub fn os_to_errorkind(os: RawError) -> ErrorKind {
    match os {
        errno::EINTR => ErrorKind::Interrupted,
        // EAGAIN and EWOULDBLOCK are the same thing
        errno::EAGAIN | errno::EWOULDBLOCK => ErrorKind::WouldBlock,
        // TODO: add other errnos
        _ => ErrorKind::Other,
    }
}

pub fn gai_strerror(code: c_int) -> String {
    unsafe {
        let ptr = libc::gai_strerror(code);
        OsStr::from_ptr(ptr).to_utf8_lossy()
    }
}
