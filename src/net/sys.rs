#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};
use crate::ffi::*;

crate::cfg_if! {
    if #[cfg(windows)] {
        pub type Socket = crate::ffi::SOCKET;
        pub const INVALID_SOCKET: Socket = usize::MAX;
    } else if #[cfg(unix)] {
        pub type Socket = crate::ffi::c_int;
        pub const INVALID_SOCKET: Socket = -1;
    }
}

// socklen_t is tricky
crate::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub type socklen_t = u32;
    } else if #[cfg(target_os = "windows")] {
        pub type socklen_t = i32;
    } else if #[cfg(target_os = "android")] {
        #[cfg(target_pointer_width = "32")]
        pub type socklen_t = i32;
        #[cfg(target_pointer_width = "64")]
        pub type socklen_t = u32;
    }
}

pub const SOMAXCONN: c_int = 128;

crate::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "android"))] {
        pub const AF_INET: c_ushort = 2;
        pub const AF_INET6: c_ushort = 10;
        pub const SOCK_STREAM: c_int = 1;
        //pub const SOCK_DGRAM: c_int = 2;
    } else if #[cfg(target_os = "windows")] {
        pub const AF_INET: c_ushort = 2;
        pub const AF_INET6: c_ushort = 23;
        pub const SOCK_STREAM: c_int = 1;
        //pub const SOCK_DGRAM: c_int = 2;
    }
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}
impl Default for sockaddr {
    fn default() -> sockaddr {
        sockaddr {
            // fill the longest field with zeroes
            _in6: sockaddr_in6 {
                sin6_family: 0,
                sin6_port: 0,
                sin6_flowinfo: 0,
                sin6_addr: in6_addr { s6_addr: [0; 16] },
                sin6_scope_id: 0,
            }
        }
    }
}

crate::cfg_if! {
    if #[cfg(unix)] {
        pub type SendLen = c_size_t;
        pub type SendRet = c_ssize_t;
    } else if #[cfg(windows)] {
        pub type SendLen = c_int;
        pub type SendRet = c_int;
    }
}

unsafe extern "C" {
    pub fn socket(domain: c_int, _type: c_int, protocol: c_int) -> Socket;
    pub fn bind(socket: Socket, sockaddr: *const sockaddr, addrlen: socklen_t) -> c_int;
    pub fn listen(socket: Socket, backlog: c_int) -> c_int;
    pub fn accept(socket: Socket, addr: *mut sockaddr, addrlen: *mut socklen_t) -> Socket;
    // These have different definition on windows
    pub fn send(socket: Socket, buf: *const u8, size: SendLen, flags: c_int) -> SendRet;
    pub fn recv(socket: Socket, buf: *mut u8, size: SendLen, flags: c_int) -> SendRet;
}

#[cfg(windows)]
unsafe extern "C" {
    pub fn WSAStartup(
        /* [in] */ wVersionRequired: WORD,
        /* [out] */ lpWSAData: LPWSADATA,
    ) -> c_int;
    pub fn closesocket(socket: Socket) -> c_int;
    pub fn WSAGetLastError() -> c_int;
}

#[cfg(windows)]
crate::block! {
    const WSADESCRIPTION_LEN: usize = 256;
    const WSASYS_STATUS_LEN: usize = 128;

    #[repr(C)]
    #[cfg(target_pointer_width = "64")]
    #[allow(non_snake_case)]
    pub struct WSAData {
        wVersion: WORD,
        wHighVersion: WORD,
        // #ifdef _WIN64
        iMaxSockets: c_ushort,
        iMaxUdpDg: c_ushort,
        lpVendorInfo: *mut c_char,
        szDescription: [c_char; WSADESCRIPTION_LEN+1],
        szSystemStatus: [c_char; WSASYS_STATUS_LEN+1],
    }

    #[cfg(target_pointer_width = "32")]
    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct WSAData {
        wVersion: WORD,
        wHighVersion: WORD,
        // #else
        szDescription: [c_char; WSADESCRIPTION_LEN+1],
        szSystemStatus: [c_char; WSASYS_STATUS_LEN+1],
        iMaxSockets: c_ushort,
        iMaxUdpDg: c_ushort,
        lpVendorInfo: *mut c_char,
    }

    pub type WSADATA = WSAData;
    pub type LPWSADATA = *mut WSADATA;
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
            return SocketAddr::V4(SocketAddrV4::new(ip, port));
        } else if family == AF_INET6 {
            let addr = unsafe { addr._in6 };
            let ip = Ipv6Addr::from_octets(addr.sin6_addr.s6_addr);
            let port = u16::from_be(addr.sin6_port);
            return SocketAddr::V6(SocketAddrV6::new(ip, port, addr.sin6_flowinfo, addr.sin6_scope_id));
        } else {
            // Fail, return fallback address...
            return SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
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
