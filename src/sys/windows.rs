#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

pub mod types {
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
    pub type PLARGE_INTEGER = *mut i64;

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

use types::*;

unsafe extern "C" {
    /// Creates or opens a file or I/O device.
    pub fn CreateFileW(
        /* [in] */ lpFileName: LPCWSTR,
        /* [in] */ dwDesiredAccess: DWORD,
        /* [in] */ dwShareMode: DWORD,
        /* [in, optional] */ lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        /* [in] */ dwCreationDisposition: DWORD,
        /* [in] */ dwFlagsAndAttributes: DWORD,
        /* [in, optional] */ hTemplateFile: HANDLE,
    ) -> HANDLE;

    /// Reads data from the specified file or input/output (I/O) device.
    pub fn ReadFile(
        /* [in] */ hFile: HANDLE,
        /* [out] */ lpBuffer: LPVOID,
        /* [in] */ nNumberOfBytesToRead: DWORD,
        /* [out, optional] */ lpNumberOfBytesRead: LPDWORD,
        /* [in, out, optional] */ lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    /// Writes data to the specified file or input/output (I/O) device.
    pub fn WriteFile(
        /* [in] */ hFile: HANDLE,
        /* [in] */ lpBuffer: LPCVOID,
        /* [in] */ nNumberOfBytesToWrite: DWORD,
        /* [out, optional] */ lpNumberOfBytesWritten: LPDWORD,
        /* [in, out, optional] */ lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;

    /// Moves the file pointer of the specified file.
    pub fn SetFilePointerEx(
        /* [in] */ hFile: HANDLE,
        /* [in] */ liDistanceToMove: LARGE_INTEGER,
        /* [out, optional] */ lpNewFilePointer: PLARGE_INTEGER,
        /* [in] */ dwMoveMethod: DWORD,
    ) -> BOOL;

    /// Closes an open object handle.
    pub fn CloseHandle(
        /* [in] */ hObject: HANDLE
    ) -> BOOL;
}

pub mod winsock2 {
    use core::ffi::{c_int, c_char, c_void, c_ushort, c_ulong, c_uint};
    use super::types::*;

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
    }

    pub const WSA_FLAG_NO_HANDLE_INHERIT: DWORD = 0x80;
    pub const FIONBIO: c_ulong = 0x8004667e;
}
