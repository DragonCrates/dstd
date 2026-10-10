use core::ffi::*;
use super::minwindef::*;
use crate::sys::addr::sockaddr;

pub const INVALID_SOCKET: SOCKET = -1_isize as SOCKET;
pub const SOCKET_ERROR: c_int = -1;

pub const SOMAXCONN: c_int = 0x7fffffff;

pub type socklen_t = i32;

pub const AF_INET: c_int = 2;
pub const AF_INET6: c_int = 23;
pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;

pub const WSADESCRIPTION_LEN: usize = 256;
pub const WSASYS_STATUS_LEN: usize = 128;

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

#[repr(C)]
#[cfg(target_pointer_width = "32")]
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

pub type GROUP = c_uint;
pub type LPWSAPROTOCOL_INFOW = *mut c_void;

unsafe extern "C" {
    pub fn WSAStartup(
        /* [in] */ wVersionRequired: WORD,
        /* [out] */ lpWSAData: LPWSADATA,
    ) -> c_int;
    pub fn WSASocketW(
        /* [in] */ af: c_int,
        /* [in] */ _type: c_int,
        /* [in] */ protocol: c_int,
        /* [in] */ lpProtocolInfo: LPWSAPROTOCOL_INFOW,
        /* [in] */ g: GROUP,
        /* [in] */ dwFlags: DWORD,
    ) -> SOCKET;
    pub fn ioctlsocket(
        /* [in] */ s: SOCKET,
        /* [in] */ cmd: c_ulong, // was long, but actually it is ulong
        /* [in, out] */ argp: *mut c_ulong,
    ) -> c_int;
    pub fn WSAGetLastError() -> c_int;
    pub fn closesocket(socket: SOCKET) -> c_int;

    /// The GetAddrInfoW function provides protocol-independent translation from a Unicode host name to an address.
    pub fn GetAddrInfoW(
        /* [in, optional] */ pNodeName: PCWSTR,
        /* [in, optional] */ pServiceName: PCWSTR,
        /* [in, optional] */ pHints: *const ADDRINFOW,
        /* [out]          */ ppResult: *mut PADDRINFOW,
    ) -> INT;

    /// The FreeAddrInfoW function frees address information that the GetAddrInfoW function dynamically allocates in addrinfoW structures.
    pub fn FreeAddrInfoW(
        /* [in] */ pAddrInfo: PADDRINFOW
    );
}

pub const WSA_FLAG_NO_HANDLE_INHERIT: DWORD = 0x80;
pub const FIONBIO: c_ulong = 0x8004667e;

unsafe extern "C" {
    //pub fn socket(domain: c_int, _type: c_int, protocol: c_int) -> SOCKET;
    pub fn bind(s: SOCKET, sockaddr: *const sockaddr, addrlen: socklen_t) -> c_int;
    pub fn listen(s: SOCKET, backlog: c_int) -> c_int;
    pub fn accept(s: SOCKET, addr: *mut sockaddr, addrlen: *mut socklen_t) -> SOCKET;
    pub fn send(s: SOCKET, buf: *const u8, size: c_int, flags: c_int) -> c_int;
    pub fn sendto(s: SOCKET, buf: *const u8, size: c_int, flags: c_int, to: *const sockaddr, tolen: c_int) -> c_int;
    pub fn recv(s: SOCKET, buf: *mut u8, size: c_int, flags: c_int) -> c_int;
    pub fn recvfrom(s: SOCKET, buf: *mut u8, size: c_int, flags: c_int, from: *mut sockaddr, fromlen: c_int) -> c_int;
    pub fn connect(s: SOCKET, name: *const sockaddr, namelen: c_int) -> c_int;
}

#[repr(C)]
#[derive(Default)]
pub struct ADDRINFOW {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: usize,
    pub ai_canonname: PWSTR,
    pub ai_addr: *mut sockaddr,
    pub ai_next: *mut ADDRINFOW,
}
pub type PADDRINFOW = *mut ADDRINFOW;
