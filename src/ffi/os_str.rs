use core::slice;
use core::fmt;
use core::error::Error;
use core::mem;
use core::ops::Deref;

extern crate alloc;
use alloc::string::{FromUtf8Error, FromUtf16Error};
use alloc::borrow::{ToOwned, Cow, Borrow};
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

#[cfg(windows)]
type _OsChar = u16;
#[cfg(unix)]
type _OsChar = u8;
/// Character that uses platform-dependent encoding
/// - On Windows, it is `u16`
/// - On Unix platforms, it is `u8`
pub type OsChar = _OsChar;

/// Reference to a platform-native ffi string
/// - On Unix systems, strings are arbitraty byte sequences that are expected to use at least partially valid UTF-8 encoding
/// - On Windows, native strings use UTF-16 encoding that may contain invalid code points
/// - In Rust, [`String`]s are always valid UTF-8 and may contain zeros
///
/// Unlike std's `OsStr`, this type DOES NOT contain a superset of UTF-8. It always uses encoding provided by current platform and can be cast to `*const u8` (unix) or `LPCWSTR` (windows) without conversion. It is always null-terminated
// As in std, repr(transparent) is an implementation detail needed for conversion from &[u8], and is not a stable API guarantee
#[repr(transparent)]
pub struct OsStr {
    inner: [OsChar],
}

impl OsStr {
    /// Constructs a new `OsStr` from a given slice
    /// # Safety
    /// Slice has to be zero terminated. If it contains other zeroes, it will be truncated - it is not a memory safety error, but is a severe logic error. You should almost always use the [`OsStr::new`] method
    pub const unsafe fn new_unchecked(slice: &[OsChar]) -> &OsStr {
        // safe as long as it is repr(transparent)
        unsafe { mem::transmute(slice) }
    }

    /// Constructs a new `OsStr` from a given slice, checking that a slice is valid
    pub fn new(slice: &[OsChar]) -> Result<&OsStr, OsStrError> {
        if slice.is_empty() { return Err(OsStrError::SliceEmpty); }
        let last = slice.len() - 1;
        if slice[last] != 0 { return Err(OsStrError::NotTerminated); }
        for &c in &slice[..last] {
            if c == 0 { return Err(OsStrError::HasZeroes); }
        }
        // Everything clear
        Ok(unsafe { OsStr::new_unchecked(slice) })
    }

    /// Constructs a new `OsStr` from a given pointer to a null-terminated multibyte string (unix) or null-terminated wide string (windows)
    /// # Safety
    /// Pointer must contain a valid null-terminated string in platform encoding - `*char` on unix and `*wchar_t` on windows. Also make sure it didn't contain any zeroes before the null terminator - it is a logic error
    pub const unsafe fn from_ptr<'a>(ptr: *const OsChar) -> &'a OsStr {
        let mut i = ptr;
        unsafe {
            // Find the null terminator
            while *i != 0 { i = i.add(1); }
        }
        let len = unsafe { i.offset_from(ptr) };
        // add null terminator
        let len = len + 1;
        let slice = unsafe { slice::from_raw_parts(ptr, len as usize) };
        unsafe { OsStr::new_unchecked(slice) }
    }

    /// Converts this `OsStr` into a `*char` or `*wchar_t` pointer (depending on platform)
    pub const fn as_ptr(&self) -> *const OsChar {
        self.inner.as_ptr()
    }

    /// Decodes the `OsStr` into native [`String`], returning error if it contained any invalid code points
    pub fn to_utf8(&self) -> Result<String, OsUtf8Error> {
        #[cfg(windows)]
        return Ok(String::from_utf16(self.as_bytes())?);
        #[cfg(unix)]
        return Ok(String::from_utf8(self.as_bytes().to_vec())?)
    }

    /// Decodes the `OsStr` into native [`String`], replacing any invalid characters
    pub fn to_utf8_lossy(&self) -> String {
        #[cfg(windows)]
        return String::from_utf16_lossy(self.as_bytes());
        #[cfg(unix)]
        return String::from_utf8_lossy(self.as_bytes()).into_owned()
    }

    /// Returns string contents without the null terminator
    pub fn as_bytes(&self) -> &[OsChar] {
        let len = self.inner.len();
        &self.inner[..len-1]
    }

    /// Returns full string contents
    pub fn as_bytes_with_nul(&self) -> &[OsChar] {
        &self.inner
    }

    /// Converts this `OsStr` into an owned [`OsString`]
    fn to_os_string(&self) -> OsString {
        unsafe { OsString::new_unchecked(self.inner.to_vec()) }
    }
}

impl ToOwned for OsStr {
    type Owned = OsString;

    fn to_owned(&self) -> OsString {
        self.to_os_string()
    }

    fn clone_into(&self, dest: &mut OsString) {
        dest.inner.clear();
        dest.inner.extend_from_slice(&self.inner);
    }
}

/*/// Escapes invalid UTF-8 sequences in a byte string (instead of replacing with U+FFFD)
fn escape_to_utf8(s: &[u8]) -> String {
    let mut out = String::with_capacity(s.len()*2);
    for chunk in s.utf8_chunks() {
        for ch in chunk.valid().chars() {
            // also filter out ASCII control characters
            if ch < ' ' {
                write!(&mut out, "\\x{:02x}", ch as u8).unwrap();
            } else {
                out.push(ch);
            }
        }
        for byte in chunk.invalid() {
            write!(&mut out, "\\x{:02x}", byte).unwrap();
        }
    }
    out
}*/

impl fmt::Debug for OsStr {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!("stalled until wtf-8 encoder is implemented")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OsStrError {
    SliceEmpty,
    NotTerminated,
    HasZeroes
}

impl fmt::Display for OsStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsStrError::SliceEmpty => f.write_str("empty slice"),
            OsStrError::NotTerminated => f.write_str("c string is not null terminated"),
            OsStrError::HasZeroes => f.write_str("c string has zeroes before null terminator"),
        }
    }
}

impl Error for OsStrError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum OsUtf8Error {
    Utf8(FromUtf8Error),
    Utf16(FromUtf16Error),
}

impl From<FromUtf8Error> for OsUtf8Error {
    fn from(err: FromUtf8Error) -> OsUtf8Error {
        OsUtf8Error::Utf8(err)
    }
}

impl From<FromUtf16Error> for OsUtf8Error {
    fn from(err: FromUtf16Error) -> OsUtf8Error {
        OsUtf8Error::Utf16(err)
    }
}

impl fmt::Display for OsUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsUtf8Error::Utf8(err) => err.fmt(f),
            OsUtf8Error::Utf16(err) => err.fmt(f),
        }
    }
}

impl Error for OsUtf8Error {}

/// Constructs a new OsStr reference by copying into a provided buffer
///
/// If buffer size was not enough, it allocates and returns `Cow::Owned`
#[allow(clippy::int_plus_one)]
#[inline]
pub fn str_to_os<'a>(s: &str, buf: &'a mut [OsChar]) -> Result<Cow<'a, OsStr>, OsStrError> {
    #[cfg(unix)]
    let len = s.len();
    #[cfg(windows)]
    let len = s.encode_utf16().count();

    #[cfg(unix)]
    let fill = |buf: &mut [u8]| {
        buf[..len].copy_from_slice(s.as_bytes());
        buf[len] = 0;
    };
    #[cfg(windows)]
    let fill = |buf: &mut [u16]| {
        for (i, c) in s.encode_utf16().enumerate() {
            buf[i] = c;
        }
        buf[len] = 0;
    };

    if len+1 <= buf.len() {
        fill(buf);
        let os_str = OsStr::new(&buf[..len+1])?;
        Ok(Cow::Borrowed(os_str))
    } else {
        let mut buf = vec![0; len+1];
        fill(&mut buf);
        let os_string = OsString::new(buf)?;
        Ok(Cow::Owned(os_string))
    }
}

/// An owned [`OsStr`]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct OsString {
    inner: Vec<OsChar>,
}

impl OsString {
    /// Constructs a new `OsString` from a given character vector
    /// # Safety
    /// Same as of [`OsStr`], last element has to be zero, and must not contain any other zeroes
    pub unsafe fn new_unchecked(v: Vec<OsChar>) -> OsString {
        OsString { inner: v }
    }

    /// Constructs a new `OsString` from a given character vector, additionally checking it for validity. Last element must be zero, should not contain any other zeroes
    pub fn new(v: Vec<OsChar>) -> Result<OsString, OsStrError> {
        // check
        let _ = OsStr::new(&v)?;
        // valid, ok
        Ok(unsafe { OsString::new_unchecked(v) })
    }

    /// Converts this `OsString` into an OsStr reference
    pub fn as_os_str(&self) -> &OsStr {
        unsafe { OsStr::new_unchecked(&self.inner) }
    }
}

impl AsRef<OsStr> for OsString {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl Borrow<OsStr> for OsString {
    fn borrow(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl Deref for OsString {
    type Target = OsStr;
    fn deref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl fmt::Debug for OsString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_os_str().fmt(f)
    }
}
