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
