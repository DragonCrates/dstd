use core::fmt;
use core::str::Utf8Error;
use core::ffi::c_int;

use crate::os_str::OsStrError;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

/// Raw OS error type. `c_int` on Linux and `DWORD` on Windows
pub type RawError = sys::RawError;

/// The error of any I/O operations
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Error {
    // will need repr_bitpacked if we add a custom error type
    pub(crate) repr: Repr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    UnexpectedEof,
    WriteZero,
    Utf8,
    OsStr(OsStrError),
    Os(RawError),
    AddrInfo(c_int),
    NoAddresses,
}

impl Error {
    /// Retrieves the last OS error
    pub fn last_os_error() -> Error {
        Error { repr: Repr::Os(sys::last_os_error()) }
    }

    /// Constructs a new error from a raw OS error
    pub fn from_raw_os_error(code: RawError) -> Error {
        Error { repr: Repr::Os(code) }
    }

    /// Returns the corresponding [`ErrorKind`] for this error
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::UnexpectedEof => ErrorKind::UnexpectedEof,
            Repr::WriteZero => ErrorKind::WriteZero,
            Repr::Utf8 | Repr::OsStr(_) => ErrorKind::InvalidData,
            Repr::Os(os) => sys::os_to_errorkind(os),
            Repr::AddrInfo(_) => ErrorKind::Other,
            Repr::NoAddresses => ErrorKind::InvalidInput,
        }
    }

    /// Returns the container raw OS error if it was one
    /// # Example
    /// ```
    /// # use dstd::io::Error;
    /// let errno = Error::last_os_error().raw_os_error().unwrap();
    /// println!("Errno: {errno}");
    /// ```
    pub fn raw_os_error(&self) -> Option<RawError> {
        match self.repr {
            Repr::Os(err) => Some(err),
            _ => None,
        }
    }

    pub(crate) fn new_gai(code: c_int) -> Error {
        Error { repr: Repr::AddrInfo(code) }
    }

    pub(crate) fn new_no_addresses() -> Error {
        Error { repr: Repr::NoAddresses }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::UnexpectedEof => f.debug_struct("UnexpectedEof").finish(),
            Repr::WriteZero => f.debug_struct("WriteZero").finish(),
            Repr::Utf8 => f.debug_struct("Utf8").finish(),
            Repr::OsStr(err) => err.fmt(f),
            Repr::Os(errno) => f.debug_struct("Os")
                .field("code", &errno)
                .field("msg", &sys::strerror(errno))
                .finish(),
            Repr::AddrInfo(code) => f.debug_struct("AddrInfo")
                .field("code", &code)
                .field("msg", &sys::gai_strerror(code))
                .finish(),
            Repr::NoAddresses => f.debug_struct("NoAddresses").finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::UnexpectedEof => f.write_str("unexpected eof"),
            Repr::WriteZero => f.write_str("write zero"),
            Repr::Utf8 => f.write_str("stream did not contain valid UTF-8"),
            Repr::OsStr(err) => err.fmt(f),
            Repr::Os(errno) => f.write_str(&sys::strerror(errno)),
            Repr::AddrInfo(code) => f.write_str(&sys::gai_strerror(code)),
            Repr::NoAddresses => f.write_str("could not resolve to any addresses"),
        }
    }
}

impl core::error::Error for Error {}

impl From<Utf8Error> for Error {
    fn from(_value: Utf8Error) -> Error {
        Error {
            repr: Repr::Utf8
        }
    }
}


impl From<OsStrError> for Error {
    fn from(value: OsStrError) -> Error {
        Error {
            repr: Repr::OsStr(value)
        }
    }
}

/// Result type alias for I/O operations
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    // dstd errors
    UnexpectedEof,
    WriteZero,
    InvalidInput,
    InvalidData,

    // os errors
    Interrupted,
    WouldBlock,

    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedEof => f.write_str("unexpected eof"),
            ErrorKind::WriteZero => f.write_str("write zero"),
            ErrorKind::InvalidInput => f.write_str("invalid input"),
            ErrorKind::InvalidData => f.write_str("invalid data"),
            ErrorKind::Interrupted => f.write_str("interrupted"),
            ErrorKind::WouldBlock => f.write_str("would block"),
            ErrorKind::Other => f.write_str("other"),
        }
    }
}

impl core::error::Error for ErrorKind {}
