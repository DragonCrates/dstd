extern crate alloc;
use alloc::borrow::Cow;
use alloc::string::String;

use crate::os_str::{OsChar, OsStr, OsStrError};

pub enum Path<'a> {
    Str(&'a str),
    OsStr(&'a OsStr),
}

impl<'a> Path<'a> {
    pub fn to_os_with(&self, buf: &'a mut [OsChar]) -> Result<Cow<'a, OsStr>, OsStrError> {
        match self {
            Path::Str(s) => OsStr::from_str_with(s, buf),
            Path::OsStr(os) => Ok(Cow::Borrowed(os)),
        }
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Path<'a> {
        Path::Str(value)
    }
}

impl<'a> From<&'a String> for Path<'a> {
    fn from(value: &'a String) -> Path<'a> {
        Path::Str(value.as_str())
    }
}
