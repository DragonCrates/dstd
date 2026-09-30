use core::ffi::{c_int, c_ushort};

use crate::sys::libc::{c_ssize_t, c_size_t};
use crate::sys::libc::fcntl;
use crate::io::Error;

pub type Socket = c_int;
pub const INVALID_SOCKET: Socket = -1;

#[cfg(target_os = "linux")]
pub type socklen_t = u32;

#[cfg(target_os = "android")]
crate::block! {
    #[cfg(target_pointer_width = "32")]
    pub type socklen_t = i32;
    #[cfg(target_pointer_width = "64")]
    pub type socklen_t = u32;
}

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    pub const AF_INET: c_ushort = 2;
    pub const AF_INET6: c_ushort = 10;
    pub const SOCK_STREAM: c_int = 1;
    //pub const SOCK_DGRAM: c_int = 2;
}

pub type SendLen = c_size_t;
pub type SendRet = c_ssize_t;

pub fn init() {}

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    const SOCK_CLOEXEC: c_int = fcntl::O_CLOEXEC;
    const SOCK_NONBLOCK: c_int = fcntl::O_NONBLOCK;
}

unsafe extern "C" {
    fn socket(domain: c_int, _type: c_int, protocol: c_int) -> Socket;
}

pub fn new_cloexec(domain: c_int, socket_type: c_int) -> Socket {
    unsafe { socket(domain, socket_type | SOCK_CLOEXEC, 0) }
}

pub fn new_nonblock(domain: c_int, socket_type: c_int) -> Socket {
    unsafe { socket(domain, socket_type | SOCK_NONBLOCK | SOCK_CLOEXEC, 0) }
}

pub fn socketerror() -> Error {
    Error::last_os_error()
}

pub use crate::sys::libc::close as closesocket;
