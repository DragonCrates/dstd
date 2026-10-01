use core::ffi::c_int;

// Linux socklen_t
#[cfg(target_os = "linux")]
pub type socklen_t = u32;

// Android socklen_t
#[cfg(target_os = "android")]
crate::block! {
    #[cfg(target_pointer_width = "32")]
    pub type socklen_t = i32;
    #[cfg(target_pointer_width = "64")]
    pub type socklen_t = u32;
}

pub const AF_INET: c_int = 2;
pub const AF_INET6: c_int = 10;

pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;

use crate::sys::libc::fcntl;
pub const SOCK_CLOEXEC: c_int = fcntl::O_CLOEXEC;
pub const SOCK_NONBLOCK: c_int = fcntl::O_NONBLOCK;

pub const SOL_SOCKET: c_int = 1;
pub const SO_REUSEPORT: c_int = 15;
