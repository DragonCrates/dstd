use core::fmt;

extern crate alloc;
use alloc::string::String;

use crate::io::{Result, Read, Write};
use crate::sync::Mutex;

mod buf;
use buf::Buf;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows::{raw_stdin, raw_stdout, raw_stderr};
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix::{raw_stdin, raw_stdout, raw_stderr};
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

fn print_internal(out: &mut impl Write, args: fmt::Arguments) {
    static BUF: Mutex<String> = Mutex::new(String::new());

    if let Some(s) = args.as_str() {
        let _ = out.write(s.as_bytes());
    } else {
        let mut buf = BUF.lock();
        buf.clear();
        fmt::write(&mut *buf, args).expect("Display implementation returned an unexpected error");
        let _ = out.write_all(buf.as_bytes());
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

struct BufferedStdin {
    buf: Buf<256>,
}

static BUFFERED_STDIN: Mutex<BufferedStdin> = Mutex::new(BufferedStdin { buf: Buf::new() });

// TODO:
// 1) does not handle partial utf-8 reads at all
// 2) probably can be decoupled into BufRead
impl BufferedStdin {
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let mut nw = 0;

        // Try to empty buffer
        if !self.buf.is_empty() {
            if let Some(newline) = self.buf.iter().position(|&i| i == b'\n') {
                buf.push_str(str::from_utf8(&self.buf[..newline+1])?);
                nw += newline+1;
                self.buf.consume(newline+1);
                return Ok(nw);
            } else {
                buf.push_str(str::from_utf8(&self.buf)?);
                nw += self.buf.buflen();
                self.buf.clear();
            }
        }

        loop {
            // Fill buf
            self.buf.resize(self.buf.maxlen());
            let nr = raw_stdin().read(&mut self.buf[..])?;
            if nr == 0 { return Ok(nw); }
            self.buf.resize(nr);

            // Try to find \n
            if let Some(newline) = self.buf.iter().position(|&i| i == b'\n') {
                buf.push_str(str::from_utf8(&self.buf[..newline+1])?);
                nw += newline+1;
                self.buf.consume(newline+1);
                return Ok(nw);
            } else {
                buf.push_str(str::from_utf8(&self.buf)?);
                nw += self.buf.buflen();
                self.buf.clear();
            }
        }
    }
}

impl Read for BufferedStdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut nw = 0;
        // Empty buffer, if it had something
        if !self.buf.is_empty() {
            if buf.len() <= self.buf.buflen() {
                buf.copy_from_slice(&self.buf[..buf.len()]);
                self.buf.consume(buf.len());
                return Ok(buf.len());
            } else {
                nw = self.buf.buflen();
                buf[..nw].copy_from_slice(&self.buf);
                self.buf.clear();
            }
        }
        // Read leftover into the buffer
        nw += raw_stdin().read(&mut buf[nw..])?;
        Ok(nw)
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
