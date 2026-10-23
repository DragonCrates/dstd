use core::ffi::c_char;

pub type CHAR = c_char;
pub type LPCCH = *const CHAR;
pub type LPSTR = *mut CHAR;

pub type WCHAR = u16;
pub type LPCWCH = *const WCHAR;

pub type PCWSTR = *const WCHAR;
pub type LPCWSTR = *const WCHAR;
pub type PWSTR = *mut WCHAR;
pub type LPWSTR = *mut WCHAR;
