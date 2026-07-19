pub mod types {
    #![allow(non_camel_case_types)]

    use core::ffi::*;

    pub type BOOL = u8;
    pub type ULONG = c_ulong;
    pub type ULONG_PTR = usize;
    pub type SHORT = c_short;
    pub type WORD = c_ushort;
    pub type DWORD = ULONG;
    pub type DWORD_PTR = ULONG_PTR;
    pub type LPDWORD = *mut DWORD;
    pub type VOID = c_void;
    pub type PVOID = *mut VOID;
    pub type LPVOID = PVOID;
    pub type LPCVOID = *const VOID;
    pub type UINT = c_uint;
    pub type UINT_PTR = usize;
    pub type LPCCH = *const c_char;
    pub type LPWSTR = *const u16;
    pub type LPCWSTR = *const u16;
    pub type WCHAR = u16;
    pub type SIZE_T = ULONG_PTR;
    pub type LARGE_INTEGER = i64;

    pub type errno_t = c_int;

    pub type HANDLE = PVOID;
    pub type HWND = *mut c_void;
    pub type SOCKET = UINT_PTR;

    pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE; // -1

    // not used
    pub type LPOVERLAPPED = *mut c_void;
    pub type LPSECURITY_ATTRIBUTES = *mut c_void;

    // TODO: move console structures here
}
