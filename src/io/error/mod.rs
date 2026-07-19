use core::fmt;
use core::str::Utf8Error;

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

// An ErrorKind type will be added in future, to match on returned OS error

/// Raw OS error type. `c_int` on Linux and `DWORD` on Windows
pub type RawError = sys::RawError;

/// The error of any I/O operations
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub(crate) repr: Repr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    UnexpectedEof,
    WriteZero,
    Utf8,
    Os(RawError),
    //AddrInfo(AddrInfoError)
}

impl Error {
    /// Retrieves the last OS error
    pub fn last_os_error() -> Error {
        return Error { repr: Repr::Os(sys::last_os_error()) };
    }

    /// Constructs a new error from a raw OS error
    pub fn from_raw_os_error(code: RawError) -> Error {
        Error { repr: Repr::Os(code) }
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
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::UnexpectedEof => f.debug_struct("UnexpectedEof").finish(),
            Repr::WriteZero => f.debug_struct("WriteZero").finish(),
            Repr::Utf8 => f.debug_struct("Utf8").finish(),
            Repr::Os(errno) => f.debug_struct("Os")
                .field("code", &errno)
                .field("msg", &sys::strerror(errno))
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
            Repr::Utf8 => f.write_str("stream did not contain valid UTF-8"),
            Repr::Os(errno) => f.write_str(&sys::strerror(errno)),
            //Repr::AddrInfo(err) => f.fmt(err),
        }
    }
}

impl core::error::Error for Error {}

impl From<Utf8Error> for Error {
    fn from(_f: Utf8Error) -> Error {
        Error {
            repr: Repr::Utf8
        }
    }
}

/// Result type alias for I/O operations
pub type Result<T> = core::result::Result<T, Error>;
