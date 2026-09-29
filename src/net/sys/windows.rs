use core::mem;
use core::ffi::{c_int, c_ushort, c_char};

use crate::sys::windows::types::*;
use crate::io::Error;

pub type Socket = SOCKET;
pub const INVALID_SOCKET: Socket = usize::MAX;

pub type socklen_t = i32;

pub const SOMAXCONN: c_int = 128;

pub const AF_INET: c_ushort = 2;
pub const AF_INET6: c_ushort = 23;
pub const SOCK_STREAM: c_int = 1;
//pub const SOCK_DGRAM: c_int = 2;

pub type SendLen = c_int;
pub type SendRet = c_int;

#[cfg(windows)]
unsafe extern "C" {
    pub fn WSAStartup(
        /* [in] */ wVersionRequired: WORD,
        /* [out] */ lpWSAData: LPWSADATA,
    ) -> c_int;
    pub fn closesocket(socket: Socket) -> c_int;
    pub fn WSAGetLastError() -> c_int;
}

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

pub fn socketerror() -> Error {
    Error::from_raw_os_error(unsafe { WSAGetLastError() as DWORD })
}
