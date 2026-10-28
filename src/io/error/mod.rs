use core::fmt::{self, Debug, Display};
use core::str::Utf8Error;
use core::ptr::NonNull;

extern crate alloc;
use alloc::boxed::Box;

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

mod ai_error;
pub(crate) use ai_error::AiError;
mod kind;
pub use kind::ErrorKind;
mod repr;
use repr::Repr;

/// Raw OS error type. `c_int` on Linux and `DWORD` on Windows
pub type RawError = sys::RawError;

pub fn last_os_error() -> RawError {
    sys::last_os_error()
}

/// Result type alias for I/O operations
pub type Result<T> = core::result::Result<T, Error>;

/// The error of any I/O operations
pub struct Error {
    repr: Repr,
}

impl Error {
    /// Constructs a new error from a raw OS error
    pub fn from_raw_os_error(code: RawError) -> Error {
        Error { repr: Repr::new_os(code) }
    }

    /// Retrieves the last OS error
    pub fn last_os_error() -> Error {
        Error::from_raw_os_error(last_os_error())
    }

    /// Returns the corresponding [`ErrorKind`] for this error
    pub fn kind(&self) -> ErrorKind {
        match self.repr.decode() {
            ErrorData::Os(os) => sys::os_to_errorkind(os),
            ErrorData::Simple(kind) => kind,
            ErrorData::SimpleMessage(msg) => msg.kind,
            ErrorData::Custom(custom) => custom.get().kind,
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
        match self.repr.decode() {
            ErrorData::Os(err) => Some(err),
            _ => None,
        }
    }

    /// Constructs an error from an [`ErrorKind`]
    ///
    /// This function does not allocate on heap
    pub const fn simple(kind: ErrorKind) -> Error {
        Error { repr: Repr::new_simple(kind) }
    }

    /// Constructs an error from a custom error type
    ///
    /// This function allocates on heap
    pub fn new<E>(kind: ErrorKind, err: E) -> Error
    where
        E: core::error::Error + Send + Sync + 'static
    {
        let custom_data = CustomData {
            kind,
            err: Box::new(err),
        };
        let boxed = Box::new(custom_data);
        let custom_ptr = Box::into_raw(boxed);
        let custom_nonnull = unsafe { NonNull::new_unchecked(custom_ptr) };
        let custom = Custom(custom_nonnull);
        let repr = Repr::new_custom(custom);
        Error { repr }
    }

    /// Shorthand for [`Error::new`] with [`ErrorKind::Other`]
    pub fn other<E>(err: E) -> Error
    where
        E: core::error::Error + Send + Sync + 'static
    {
        Error::new(ErrorKind::Other, err)
    }

    pub(crate) const fn from_const(msg: &'static SimpleMessage) -> Error {
        Error { repr: Repr::new_simple_message(msg) }
    }
}

impl Drop for Error {
    #[allow(clippy::single_match)]
    fn drop(&mut self) {
        match self.repr.decode() {
            ErrorData::Custom(custom) => {
                let boxed = unsafe { Box::from_raw(custom.0.as_ptr()) };
                drop(boxed);
            }
            _ => {}
        }
    }
}

enum ErrorData {
    Os(RawError),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
    Custom(Custom),
}

pub(crate) struct SimpleMessage {
    pub(crate) kind: ErrorKind,
    pub(crate) msg: &'static str,
}

struct Custom(NonNull<CustomData>);
struct CustomData {
    kind: ErrorKind,
    err: Box<dyn core::error::Error + Send + Sync + 'static>,
}

// SAFETY: all fields are Send + Sync
unsafe impl Send for CustomData {}
unsafe impl Sync for CustomData {}
unsafe impl Send for Custom {}
unsafe impl Sync for Custom {}

impl Custom {
    fn get(&self) -> &CustomData {
        unsafe { self.0.as_ref() }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr.decode() {
            ErrorData::Os(errno) => f.debug_struct("Os")
                .field("code", &errno)
                .field("msg", &sys::strerror(errno))
                .finish(),
            ErrorData::Simple(kind) => f.debug_tuple("Kind")
                .field(&kind)
                .finish(),
            ErrorData::SimpleMessage(msg) => f.debug_struct("Error")
                .field("kind", &msg.kind)
                .field("message", &msg.msg)
                .finish(),
            ErrorData::Custom(custom) => Debug::fmt(&custom.get().err, f),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr.decode() {
            ErrorData::Os(errno) => f.write_str(&sys::strerror(errno)),
            ErrorData::Simple(kind) => Display::fmt(&kind, f),
            ErrorData::SimpleMessage(msg) => f.write_str(msg.msg),
            ErrorData::Custom(custom) => Display::fmt(&custom.get().err, f),
        }
    }
}

impl core::error::Error for Error {}

impl From<Utf8Error> for Error {
    fn from(_value: Utf8Error) -> Error {
        const_error!(ErrorKind::InvalidData, "stream did not contain valid utf-8 data")
    }
}

impl From<OsStrError> for Error {
    fn from(value: OsStrError) -> Error {
        match value {
            OsStrError::NotTerminated => const_error!(ErrorKind::InvalidData, "c string was not null terminated"),
            OsStrError::HasZeroes => const_error!(ErrorKind::InvalidData, "c string has zeroes before null terminator"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Error {
        Error::simple(value)
    }
}

macro_rules! const_error {
    ($kind:expr, $msg:literal) => {{
        use $crate::io::SimpleMessage;
        const SIMPLE_MESSAGE: SimpleMessage = SimpleMessage {
            kind: $kind,
            msg: $msg,
        };
        $crate::io::Error::from_const(&SIMPLE_MESSAGE)
    }}
}
pub(crate) use const_error;
