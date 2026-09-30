use core::mem;
use core::ptr;
use core::ffi::{c_int, c_ushort, c_ulong};

use crate::sys::windows::types::*;
use crate::sys::windows::winsock2::*;
use crate::io::Error;

pub type Socket = SOCKET;
pub const INVALID_SOCKET: Socket = usize::MAX;
const SOCKET_ERROR: c_int = -1;

pub type socklen_t = i32;

pub const AF_INET: c_ushort = 2;
pub const AF_INET6: c_ushort = 23;
pub const SOCK_STREAM: c_int = 1;
//pub const SOCK_DGRAM: c_int = 2;

pub type SendLen = c_int;
pub type SendRet = c_int;

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

pub fn new_cloexec(domain: c_int, socket_type: c_int) -> Socket {
    unsafe { WSASocketW(domain, socket_type, 0, ptr::null_mut(), 0, WSA_FLAG_NO_HANDLE_INHERIT) }
}

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

pub fn socketerror() -> Error {
    Error::from_raw_os_error(unsafe { WSAGetLastError() as DWORD })
}

pub use crate::sys::windows::winsock2::closesocket;
