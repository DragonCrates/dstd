use core::ptr;
use core::ffi::c_int;

extern crate alloc;
use alloc::format;
use alloc::string::String;

use crate::sys::windows::minwindef::*;

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
    let mut buf = [0 as WCHAR; 128];

    let ret = unsafe { FormatMessageW(
        FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        ptr::null(),
        error,
        0,
        buf.as_mut_ptr(),
        buf.len() as DWORD,
        ptr::null_mut(),
    ) };

    if ret == 0 {
        return format!("Unknown error {error}");
    }

    let mut end = buf.iter().position(|&i| i == 0).expect("unterminated C string");
    if buf[end-2] == b'\r' as u16 && buf[end-1] == b'\n' as u16 {
        end -= 2;
    }

    String::from_utf16_lossy(&buf[..end])
}

pub fn os_to_errorkind(_os: DWORD) -> ErrorKind {
    ErrorKind::Other
}

pub fn gai_strerror(code: c_int) -> String {
    strerror(code as DWORD)
}
