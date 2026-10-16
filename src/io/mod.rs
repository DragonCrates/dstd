mod stdio;
pub use stdio::{Stdin, stdin, Stdout, stdout, Stderr, stderr};

mod error;
pub(crate) use error::{AiError, SimpleMessage, const_error};
pub use error::{Result, Error, ErrorKind, RawError};

mod read;
pub use read::Read;
mod write;
pub use write::Write;
mod seek;
pub use seek::{Seek, SeekFrom};

mod buf;

mod buf_read;
pub use buf_read::{BufRead, BufReader};
