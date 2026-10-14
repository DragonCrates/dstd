use core::fmt::{self, Debug, Display};
use core::error::Error;

use super::sys;

pub(crate) struct AiError(pub(crate) i32);

impl Debug for AiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AiError")
            .field("code", &self.0)
            .field("message", &sys::gai_strerror(self.0))
            .finish()
    }
}

impl Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&sys::gai_strerror(self.0))
    }
}

impl Error for AiError {}
