use core::ffi::c_int;
use crate::sys::addr::sockaddr;

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
pub const SO_REUSEADDR: c_int = 2;

pub const SOL_IPV6: c_int = 41;
pub const IPV6_V6ONLY: c_int = 26;

pub const MSG_NOSIGNAL: c_int = 0x4000;

// Glibc addrinfo
#[repr(C)]
#[derive(Default)]
#[cfg(target_os = "linux")]
pub struct addrinfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: socklen_t,
    pub ai_addr: *const sockaddr,
    pub ai_canonname: *const u8,
    pub ai_next: *const addrinfo,
}

// Bionic addrinfo
#[repr(C)]
#[derive(Default)]
#[cfg(target_os = "android")]
pub struct addrinfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: socklen_t,
    pub ai_canonname: *const u8,
    pub ai_addr: *const sockaddr,
    pub ai_next: *const addrinfo,
}
