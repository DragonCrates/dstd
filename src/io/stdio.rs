use core::fmt;

use crate::io::{self, Result, Error, Read, Write};
use crate::ffi::*;
use crate::prelude::ToString;

crate::cfg_if! {
    if #[cfg(unix)] {
        const STDIN: c_int = 0;
        const STDOUT: c_int = 1;
        const STDERR: c_int = 2;
        type StdioType = c_int;
    } else if #[cfg(windows)] {
        // #define STD_INPUT_HANDLE ((DWORD)-10)
        const STDIN: DWORD = u32::MAX-9;
        // #define STD_OUTPUT_HANDLE ((DWORD)-11)
        const STDOUT: DWORD = u32::MAX-10;
        // #define STD_ERROR_HANDLE ((DWORD)-12)
        pub(crate) const STDERR: DWORD = u32::MAX-11;
        type StdioType = DWORD;
    }
}

#[cfg(windows)]
unsafe extern "C" {
    /// Retrieves a handle to the specified standard device (standard input, standard output, or standard error).
    pub(crate) fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    /// Retrieves the current input mode of a console's input buffer or the current output mode of a console screen buffer.
    fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
    /// Maps a character string to a UTF-16 (wide character) string.
    fn MultiByteToWideChar(
        /* [in] */ CodePage: UINT,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ lpMultiByteStr: LPCCH,
        /* [in] */ cbMultiByte: c_int,
        /* [out, optional */ lpWideCharStr: LPWSTR,
        /* [in] */ cchWideChar: c_int
    ) -> c_int;
    /// Writes a character string to a console screen buffer beginning at the current cursor location.
    fn WriteConsoleW(
        /* _In_ */ hConsoleOutput: HANDLE,
        /* _In_ */ lpBuffer: LPCVOID,
        /* _In_ */ nNumberOfCharsToWrite: DWORD,
        /* _Out_opt_ */ lpNumberOfCharsWritten: LPDWORD,
        /* _Reserved_ */ lpReserved: LPVOID,
    ) -> BOOL;
}

/// Returns a handle to the standard input of the current process
pub fn stdin() -> Stdio { Stdio(STDIN) }
/// Returns a handle to the standard output of the current process
pub fn stdout() -> Stdio { Stdio(STDOUT) }
/// Returns a handle to the standard error of the current process
pub fn stderr() -> Stdio { Stdio(STDERR) }

/// A handle to the stdio stream of the process. Returned by [`stdin`], [`stdout`] or [`stderr`] functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stdio(StdioType);

// TODO buffered stdin with global locking

impl Read for Stdio {
    #[cfg(unix)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe { io::read(self.0, buf.as_mut_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    // TODO windows
    #[cfg(windows)]
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
}

impl Write for Stdio {
    #[cfg(unix)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe { io::write(self.0, buf.as_ptr(), buf.len()) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    #[cfg(windows)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        use core::ptr;
        use crate::prelude::vec;

        let handle = unsafe { GetStdHandle(self.0) };
        if handle.is_null() { return Ok(0); }
        if handle == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }
        let mut mode: DWORD = 0;
        let ret = unsafe { GetConsoleMode(handle, &mut mode as LPDWORD) };
        // TODO: if console codepage is utf-8, skip too
        if ret != 0 {
            // Console
            let mut wstr = vec![0_u16; buf.len()];
            let ret = unsafe { MultiByteToWideChar(
                65001, // CodePage (CP_UTF8)
                0, // dwFlags
                buf.as_ptr() as LPCCH, // lpMultiByteStr
                buf.len() as c_int, // cbMultiByte
                wstr.as_mut_ptr(), // lpWideCharStr
                wstr.len() as c_int, // cchWideChR
            ) };
            if ret == 0 { return Err(Error::last_os_error()); }
            let mut nw: DWORD = 0;
            let ret = unsafe { WriteConsoleW(
                handle, // hConsoleOutput
                wstr.as_ptr() as LPCVOID, // lpBuffer
                wstr.len() as DWORD, // nNumberOfCharsToWrite
                &mut nw as LPDWORD, // lpNumberOfCharsWritten
                ptr::null_mut(), // lpReserved
            ) };
            if ret == 0 { return Err(Error::last_os_error()); }
            Ok(nw as usize)
        } else {
            // Redirect
            let mut nw: DWORD = 0;
            let ret = unsafe { io::WriteFile(
                handle, // hFile
                buf.as_ptr() as LPCVOID, // lpBuffer
                buf.len() as DWORD, // nNumberOfBytesToWrite
                &mut nw as LPDWORD, // lpNumberOfBytesWritten
                ptr::null_mut(), // lpOverlapped
            ) };
            if ret == 0 { return Err(Error::last_os_error()); }
            Ok(nw as usize)
        }
    }
}

impl Stdio {
    #[doc(hidden)]
    /// Not a public API! Please use the `println!` macro instead
    pub fn __print_internal(&mut self, args: fmt::Arguments) {
        if let Some(s) = args.as_str() {
            let _ = self.write(s.as_bytes());
        } else {
            let _ = self.write(args.to_string().as_bytes());
        }
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
        println!("");
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
        eprintln!("")
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
