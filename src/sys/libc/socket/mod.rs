use core::ffi::{c_int, c_void};
use crate::sys::addr::sockaddr;
use crate::sys::libc::{c_size_t, c_ssize_t};

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    mod linux;
    pub use linux::*;
}

unsafe extern "C" {
    pub fn socket(domain: c_int, _type: c_int, protocol: c_int) -> c_int;
    pub fn bind(sockfd: c_int, sockaddr: *const sockaddr, addrlen: socklen_t) -> c_int;
    pub fn listen(sockfd: c_int, backlog: c_int) -> c_int;
    pub fn accept4(sockfd: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t, flags: c_int) -> c_int;
    pub fn connect(sockfd: c_int, sockaddr: *const sockaddr, addrlen: socklen_t) -> c_int;
    pub fn send(sockfd: c_int, buf: *const u8, size: c_size_t, flags: c_int) -> c_ssize_t;
    pub fn sendto(sockfd: c_int, buf: *const u8, size: c_size_t, flags: c_int, dest_addr: *const sockaddr, addrlen: socklen_t) -> c_ssize_t;
    pub fn recv(sockfd: c_int, buf: *mut u8, size: c_size_t, flags: c_int) -> c_ssize_t;
    pub fn recvfrom(sockfd: c_int, buf: *mut u8, size: c_size_t, flags: c_int, src_addr: *mut sockaddr, addrlen: socklen_t) -> c_ssize_t;
    pub fn setsockopt(sockfd: c_int, level: c_int, optname: c_int, optval: *const c_void, optlen: socklen_t) -> c_int;
}

pub const SOMAXCONN: c_int = 128;
