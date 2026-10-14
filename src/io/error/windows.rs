use core::ptr;
use core::ffi::c_int;

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::sys::windows::minwindef::*;
use crate::sys::windows::wsa_errnos;

use super::ErrorKind;

unsafe extern "C" {
    /// Retrieves the calling thread's last-error code value.
    fn GetLastError() -> DWORD;
}

pub type RawError = DWORD;

pub fn last_os_error() -> RawError {
    unsafe { GetLastError() }
}

unsafe extern "C" {
    fn FormatMessageW(
        /* [in] */ dwFlags: DWORD,
        /* [in, optional] */ lpSource: LPCVOID,
        /* [in] */ dwMessageId: DWORD,
        /* [in] */ dwLanguageId: DWORD,
        /* [out] */ lpBuffer: LPWSTR,
        /* [in] */ nSize: DWORD,
        /* [in, optional] */ Arguments: LPVOID,
    ) -> DWORD;
}

const FORMAT_MESSAGE_IGNORE_INSERTS: DWORD = 0x00000200;
const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;

pub fn strerror(error: DWORD) -> String {
    let mut wstr = [0 as WCHAR; 128];

    let ret = unsafe { FormatMessageW(
        FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        ptr::null(),
        error,
        0,
        wstr.as_mut_ptr(),
        wstr.len() as DWORD,
        ptr::null_mut(),
    ) };

    if ret == 0 { return format!("Unknown error {error}"); }
    let mut wstr_len = ret as usize;

    if wstr_len >= 2 {
        if wstr[wstr_len-2] == b'\r' as u16 && wstr[wstr_len-1] == b'\n' as u16 {
            wstr_len -= 2;
        }
    }

    String::from_utf16_lossy(&wstr[..wstr_len])
}

pub fn os_to_errorkind(os: DWORD) -> ErrorKind {
    match os {
        wsa_errnos::WSAEINTR => ErrorKind::Interrupted,
        wsa_errnos::WSAEWOULDBLOCK => ErrorKind::WouldBlock,
        // TODO: add other errors
        _ => ErrorKind::Other,
    }
}

pub fn gai_strerror(code: c_int) -> String {
    strerror(code as DWORD)
}
