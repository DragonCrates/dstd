#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_ushort};
use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};

#[cfg(windows)]
crate::block! {
    mod windows;
    pub use windows::*;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    pub use unix::*;
}

pub type sa_family_t = c_ushort;
pub type in_addr_t = u32;
pub type in_port_t = u16;

#[repr(C)]
#[derive(Clone, Copy)]
pub union sockaddr {
    pub storage: sockaddr_storage,
    pub _in: sockaddr_in,
    pub _in6: sockaddr_in6,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t, // AF_INET
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t, // AF_INET6
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

impl Default for sockaddr {
    fn default() -> sockaddr {
        sockaddr {
            // Initialize last field
            _in6: sockaddr_in6::default()
        }
    }
}

unsafe extern "C" {
    pub fn bind(socket: Socket, sockaddr: *const sockaddr, addrlen: socklen_t) -> c_int;
    pub fn listen(socket: Socket, backlog: c_int) -> c_int;
    pub fn accept(socket: Socket, addr: *mut sockaddr, addrlen: *mut socklen_t) -> Socket;
    pub fn send(socket: Socket, buf: *const u8, size: SendLen, flags: c_int) -> SendRet;
    pub fn recv(socket: Socket, buf: *mut u8, size: SendLen, flags: c_int) -> SendRet;
}

pub trait SocketAddrExt {
    fn from_sockaddr(addr: sockaddr) -> Self;
    fn to_sockaddr(&self) -> sockaddr;
    fn address_family(&self) -> sa_family_t;
}

impl SocketAddrExt for SocketAddr {
    fn from_sockaddr(addr: sockaddr) -> SocketAddr {
        let family = unsafe { addr.storage.ss_family };
        if family == AF_INET {
            let addr = unsafe { addr._in };
            let ip = Ipv4Addr::from_octets(addr.sin_addr.s_addr.to_ne_bytes());
            let port = u16::from_be(addr.sin_port);
            SocketAddr::V4(SocketAddrV4::new(ip, port))
        } else if family == AF_INET6 {
            let addr = unsafe { addr._in6 };
            let ip = Ipv6Addr::from_octets(addr.sin6_addr.s6_addr);
            let port = u16::from_be(addr.sin6_port);
            SocketAddr::V6(SocketAddrV6::new(ip, port, addr.sin6_flowinfo, addr.sin6_scope_id))
        } else {
            // Fail, return fallback address...
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        }
    }
    fn to_sockaddr(&self) -> sockaddr {
        match self {
            SocketAddr::V4(addr) => sockaddr {
                _in: sockaddr_in {
                    sin_family: AF_INET,
                    sin_port: addr.port().to_be(),
                    // Addr is already stored as BE. We don't want byteswaps here, so from_ne_bytes
                    // Ipv4Addr::to_bits does byteswap, if you wondered
                    sin_addr: in_addr { s_addr: u32::from_ne_bytes(addr.ip().octets()) },
                }
            },
            SocketAddr::V6(addr) => sockaddr {
                _in6: sockaddr_in6 {
                    sin6_family: AF_INET6,
                    sin6_port: addr.port().to_be(),
                    sin6_flowinfo: addr.flowinfo(),
                    sin6_addr: in6_addr { s6_addr: addr.ip().octets() },
                    sin6_scope_id: addr.scope_id(),
                }
            },
        }
    }
    fn address_family(&self) -> sa_family_t {
        match self {
            SocketAddr::V4(_) => AF_INET,
            SocketAddr::V6(_) => AF_INET6,
        }
    }
}
