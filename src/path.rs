extern crate alloc;
use alloc::borrow::Cow;

use crate::os_str::{OsChar, OsStr, OsStrError};

pub trait Path {
    fn to_os_with<'a>(&'a self, buf: &'a mut [OsChar]) -> Result<Cow<'a, OsStr>, OsStrError>;
}

impl Path for &str {
    fn to_os_with<'a>(&'a self, buf: &'a mut [OsChar]) -> Result<Cow<'a, OsStr>, OsStrError> {
        OsStr::from_str_with(self, buf)
    }
}

impl Path for &OsStr {
    fn to_os_with<'a>(&'a self, _buf: &'a mut [OsChar]) -> Result<Cow<'a, OsStr>, OsStrError> {
        Ok(Cow::Borrowed(self))
    }
}
