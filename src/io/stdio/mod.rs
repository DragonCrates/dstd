use core::fmt;

extern crate alloc;
use alloc::string::String;

use crate::io::{Result, BufRead, Read, Write};
use crate::io::buf::Stack;
use crate::io::buf_read::BufReaderInner;
use crate::sync::Mutex;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows::{RawStdio, raw_stdin, raw_stdout, raw_stderr};
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix::{RawStdio, raw_stdin, raw_stdout, raw_stderr};
}

/// Returns a handle to the standard input of the current process
pub fn stdin() -> Stdin { Stdin(()) }
/// Returns a handle to the standard output of the current process
pub fn stdout() -> Stdout { Stdout(()) }
/// Returns a handle to the standard error of the current process
pub fn stderr() -> Stderr { Stderr(()) }

pub struct Stdin(());

impl Stdin {
    pub fn read_line(&self, buf: &mut String) -> Result<usize> {
        BUFFERED_STDIN.lock().read_line(buf)
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        BUFFERED_STDIN.lock().read(buf)
    }
}

struct BufferedStdin(BufReaderInner<Stack, RawStdio>);
static BUFFERED_STDIN: Mutex<BufferedStdin> = Mutex::new(BufferedStdin::new());

impl BufferedStdin {
    const fn new() -> BufferedStdin {
        let inner = BufReaderInner::const_new(raw_stdin());
        BufferedStdin(inner)
    }
}

impl Read for BufferedStdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl BufRead for BufferedStdin {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.0.consume(amt);
    }
}

fn print_internal(out: &mut impl Write, args: fmt::Arguments) {
    static BUF: Mutex<String> = Mutex::new(String::new());

    if let Some(s) = args.as_str() {
        let _ = out.write(s.as_bytes());
    } else {
        // This probably requires a reentrant mutex, and we don't have one
        if let Some(mut buf) = BUF.try_lock() {
            buf.clear();
            fmt::write(&mut *buf, args).expect("Display implementation returned an unexpected error");
            let _ = out.write_all(buf.as_bytes());
        } else {
            let mut buf = String::new();
            fmt::write(&mut buf, args).expect("Display implementation returned an unexpected error");
            let _ = out.write_all(buf.as_bytes());
        }
    }
}

pub struct Stdout(());

impl Stdout {
    #[doc(hidden)]
    /// Not a public API! Please use the `println!` macro instead
    pub fn __print_internal(&mut self, args: fmt::Arguments) {
        print_internal(self, args);
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        raw_stdout().write(buf)
    }
}

pub struct Stderr(());

impl Stderr {
    #[doc(hidden)]
    /// Not a public API! Please use the `println!` macro instead
    pub fn __print_internal(&mut self, args: fmt::Arguments) {
        print_internal(self, args);
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        raw_stderr().write(buf)
    }
}

/// Prints to the standard output
#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::io::stdout().__print_internal(format_args!($($arg)*))
    };
}

/// Prints to the standard output, with a newline
#[macro_export]
macro_rules! println {
    () => {
        $crate::println!("");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

/// Prints to the standard error
#[macro_export]
macro_rules! eprint {
    () => {};
    ($($arg:tt)*) => {
        $crate::io::stderr().__print_internal(format_args!($($arg)*))
    };
}

/// Prints to the standard error, with a newline
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprintln!("")
    };
    ($($arg:tt)*) => {
        $crate::eprint!("{}\n", format_args!($($arg)*))
    };
}

/// Prints and returns value of a given expression for quick debugging
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($msg:literal) => {{
        $crate::eprintln!("[{}:{}:{}] {}", file!(), line!(), column!(), stringify!($msg));
        $msg
    }};
    ($e:expr) => {{
        let e = $e;
        $crate::eprintln!("[{}:{}:{}] {} = {:#?}", file!(), line!(), column!(), stringify!($e), e);
        e
    }};
    ($($e:expr),+ $(,)?) => {
        ($($crate::dbg!($e)),+,)
    };
}
