use core::mem;
use core::ptr;
use core::ffi::{c_int, c_ushort, c_char, c_uint, c_void, c_ulong};

use crate::sys::windows::types::*;
use crate::io::Error;

pub type Socket = SOCKET;
pub const INVALID_SOCKET: Socket = usize::MAX;
const SOCKET_ERROR: c_int = -1;

pub type socklen_t = i32;

pub const SOMAXCONN: c_int = 128;

pub const AF_INET: c_ushort = 2;
pub const AF_INET6: c_ushort = 23;
pub const SOCK_STREAM: c_int = 1;
//pub const SOCK_DGRAM: c_int = 2;

pub type SendLen = c_int;
pub type SendRet = c_int;

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

unsafe extern "C" {
    pub fn WSAStartup(
        /* [in] */ wVersionRequired: WORD,
        /* [out] */ lpWSAData: LPWSADATA,
    ) -> c_int;
}

pub fn init() {
    use crate::sync::Once;
    static WSA_INITIALIZED: Once = Once::new();

    WSA_INITIALIZED.call_once(|| unsafe {
        let mut data: WSADATA = mem::zeroed();
        if WSAStartup(0x0202, &mut data) != 0 {
            panic!("WinSock initialization failed");
        }
    });
}

type GROUP = c_uint;
type LPWSAPROTOCOL_INFOW = *mut c_void;

unsafe extern "C" {
    fn WSASocketW(
        /* [in] */ af: c_int,
        /* [in] */ _type: c_int,
        /* [in] */ protocol: c_int,
        /* [in] */ lpProtocolInfo: LPWSAPROTOCOL_INFOW,
        /* [in] */ g: GROUP,
        /* [in] */ dwFlags: DWORD,
    ) -> SOCKET;
}

const WSA_FLAG_NO_HANDLE_INHERIT: DWORD = 0x80;

pub fn new_cloexec(domain: c_int, socket_type: c_int) -> Socket {
    unsafe { WSASocketW(domain, socket_type, 0, ptr::null_mut(), 0, WSA_FLAG_NO_HANDLE_INHERIT) }
}

unsafe extern "C" {
    fn ioctlsocket(
        /* [in] */ s: SOCKET,
        /* [in] */ cmd: c_ulong, // was long, but actually it is ulong
        /* [in, out] */ argp: *mut c_ulong,
    ) -> c_int;
}

const FIONBIO: c_ulong = 0x8004667e;

pub fn new_nonblock(domain: c_int, socket_type: c_int) -> Socket {
    let sock = unsafe { WSASocketW(domain, socket_type, 0, ptr::null_mut(), 0, WSA_FLAG_NO_HANDLE_INHERIT) };
    if sock != INVALID_SOCKET {
        let mut mode: c_ulong = 1;
        let ret = unsafe { ioctlsocket(sock, FIONBIO, &mut mode) };
        if ret == SOCKET_ERROR {
            unsafe { closesocket(sock); }
            return INVALID_SOCKET;
        }
    }
    sock
}

unsafe extern "C" {
    pub fn WSAGetLastError() -> c_int;
}

pub fn socketerror() -> Error {
    Error::from_raw_os_error(unsafe { WSAGetLastError() as DWORD })
}

unsafe extern "C" {
    pub fn closesocket(socket: Socket) -> c_int;
}
