use core::fmt;

use crate::ffi::*;
use crate::prelude::String;

// An ErrorKind type will be added in future, to match on returned OS error
// TODO: windows.rs unix.rs

#[cfg(unix)]
type _ErrorOs = c_int;
#[cfg(windows)]
type _ErrorOs = DWORD;
/// Raw OS error type. `c_int` on Linux and `DWORD` on Windows
pub type ErrorOs = _ErrorOs;

/// The error of any I/O operations
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub(crate) repr: Repr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    UnexpectedEof,
    WriteZero,
    Os(ErrorOs),
    //AddrInfo(AddrInfoError)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod errno {
    pub const EINTR: i32 = 4;
    pub const EAGAIN: i32 = 11;
}

unsafe extern "C" {
    /// Retrieves the calling thread's last-error code value.
    #[cfg(windows)]
    fn GetLastError() -> DWORD;

    #[cfg(target_vendor = "apple")]
    #[link_name = "__error"]
    fn __errno() -> *mut c_int;

    /// Returns the address of the calling thread's `errno` storage.
    #[cfg(target_os = "linux")]
    #[link_name = "__errno_location"]
    fn __errno() -> *mut c_int;

    #[cfg(target_os = "android")]
    fn __errno() -> *mut c_int;
}

impl Error {
    /// Retrieves the last OS error
    pub fn last_os_error() -> Error {
        #[cfg(windows)]
        return Error { repr: Repr::Os(unsafe { GetLastError() }) };
        #[cfg(unix)]
        return Error { repr: Repr::Os(unsafe { *__errno() }) }
    }
    /// Constructs a new error from a raw OS error
    pub fn from_raw_os_error(code: ErrorOs) -> Error {
        Error { repr: Repr::Os(code) }
    }
    /// Returns the container raw OS error if it was one
    pub fn raw_os_error(&self) -> Option<ErrorOs> {
        match self.repr {
            Repr::Os(err) => Some(err),
            _ => None,
        }
    }
}

unsafe extern "C" {
    // glibc
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[link_name = "__xpg_strerror_r"]
    fn strerror_r(errnum: c_int, buf: *mut u8, size: c_size_t) -> c_int;
    // musl, bionic
    #[cfg(any(target_os = "android", target_env = "musl"))]
    fn strerror_r(errnum: c_int, buf: *mut u8, size: c_size_t) -> c_int;
}

#[cfg(unix)]
fn strerror(errno: ErrorOs) -> String {
    let mut buf = [0_u8; 128];
    // Returns 0 on error, but buffer will still have contents after this call
    unsafe { strerror_r(errno, buf.as_mut_ptr(), buf.len()); }
    let zero = buf.iter().position(|&i| i == 0).expect("unterminated C string");
    String::from_utf8_lossy(&buf[..zero]).into()
}

#[cfg(windows)]
crate::block! {
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
}

#[cfg(windows)]
fn strerror(error: ErrorOs) -> String {
    use core::ptr;
    use crate::prelude::format;

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

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::UnexpectedEof => f.debug_struct("UnexpectedEof").finish(),
            Repr::WriteZero => f.debug_struct("WriteZero").finish(),
            Repr::Os(errno) => f.debug_struct("Os")
                .field("code", &errno)
                .field("msg", &strerror(errno))
                .finish(),
            //Repr::AddrInfo(err) => f.fmt(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::UnexpectedEof => f.write_str("unexpected eof"),
            Repr::WriteZero => f.write_str("write zero"),
            Repr::Os(errno) => f.write_str(&strerror(errno)),
            //Repr::AddrInfo(err) => f.fmt(err),
        }
    }
}

impl core::error::Error for Error {}

/// Result type alias for I/O operations
pub type Result<T> = core::result::Result<T, Error>;
