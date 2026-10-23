use core::ptr;
use core::ffi::c_int;

extern crate alloc;
use alloc::vec::Vec;

use crate::io::{Result, Error, Read, Write};
use crate::sys::windows::minwindef::*;
use crate::sys::windows::winnt::*;
use crate::sys::windows::winbase::*;
use crate::sys::windows::fileapi::*;
use crate::sys::windows::consoleapi::*;
use crate::sys::windows::stringapiset::*;
use crate::sys::windows::winnls::CP_UTF8;
use crate::sys::windows::processenv::GetStdHandle;
use crate::sys::windows::handleapi::INVALID_HANDLE_VALUE;
use crate::sys::windows::sysinfoapi::GetVersion;
use crate::sync::Mutex;

pub struct RawStdio(DWORD);

pub const fn raw_stdin() -> RawStdio {
    RawStdio(STD_INPUT_HANDLE)
}

pub fn raw_stdout() -> RawStdio {
    RawStdio(STD_OUTPUT_HANDLE)
}

pub fn raw_stderr() -> RawStdio {
    RawStdio(STD_ERROR_HANDLE)
}

impl Read for RawStdio {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let handle = unsafe { GetStdHandle(self.0) };
        if handle == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }

        // Just return eof if we don't have console window
        if handle.is_null() { return Ok(0); }

        // Check the same comment for the write function
        let codepage = unsafe { GetConsoleCP() };
        if is_console(handle) && codepage == CP_UTF8 && !is_win7() {
            // Console
            with_static_utf16_buffer(|wstr| {
                // We need amount of bytes which will fit inside `buf`
                // 1 UTF-16 code point produces at most 3 UTF-8 bytes
                wstr.resize(buf.len() / 3, 0);

                let mut nr: DWORD = 0;
                let ret = unsafe { ReadConsoleW(
                    handle, // hConsoleInput
                    wstr.as_mut_ptr() as LPVOID, // lpBuffer
                    wstr.len() as DWORD, // nNumberOfCharsToRead
                    &mut nr, // lpNumberOfCharsRead
                    ptr::null_mut(), // pInputControl
                ) };
                if ret == FALSE { return Err(Error::last_os_error()); }

                let str_len = unsafe { WideCharToMultiByte(
                    CP_UTF8, // CodePage
                    0, // dwFlags
                    wstr.as_ptr(), // lpWideCharStr
                    nr as c_int, // cchWideChar
                    buf.as_mut_ptr() as LPSTR, // lpMultiByteStr
                    buf.len() as c_int, // cbMultiByte
                    ptr::null(), // lpDefaultChar
                    ptr::null_mut(), // lpUsedDefaultChar
                ) };
                if str_len == 0 { return Err(Error::last_os_error()); }
                Ok(str_len as usize)
            })
        } else {
            // Not a console, or UTF-8 console
            let mut nr: DWORD = 0;
            let ret = unsafe { ReadFile(
                handle, // hFile
                buf.as_mut_ptr() as LPVOID, // lpBuffer
                buf.len() as DWORD, // nNumberOfBytesToRead
                &mut nr, // lpNumberOfBytesRead
                ptr::null_mut(), // lpOverlapped
            ) };
            if ret == FALSE { return Err(Error::last_os_error()); }
            Ok(nr as usize)
        }
    }
}

impl Write for RawStdio {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let handle = unsafe { GetStdHandle(self.0) };
        if handle == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }

        // Just return eof if we don't have console window
        if handle.is_null() { return Ok(0); }

        // Use WriteConsoleW if the following conditions are met:
        // - Handle is a console handle
        // If console is a file, no codepage translation is required
        // - GetConsoleOutputCP() == CP_UTF8
        // If console uses UTF-8 codepage, we can skip UTF-16 roundtrip
        // - OS version is Windows 8 (NT 6.2) or newer
        // This check is needed because UTF-8 support is broken in w7 and older
        let codepage = unsafe { GetConsoleOutputCP() };
        if is_console(handle) && codepage == CP_UTF8 && !is_win7() {
            // Console
            with_static_utf16_buffer(|wstr| {
                wstr.resize(buf.len(), 0);

                let wstr_len = unsafe { MultiByteToWideChar(
                    CP_UTF8, // CodePage
                    0, // dwFlags
                    buf.as_ptr() as LPCCH, // lpMultiByteStr
                    buf.len() as c_int, // cbMultiByte
                    wstr.as_mut_ptr(), // lpWideCharStr
                    wstr.len() as c_int, // cchWideChR
                ) };
                if wstr_len == 0 { return Err(Error::last_os_error()); }

                let mut nw: DWORD = 0;
                let ret = unsafe { WriteConsoleW(
                    handle, // hConsoleOutput
                    wstr.as_ptr() as LPCVOID, // lpBuffer
                    wstr_len as DWORD, // nNumberOfCharsToWrite
                    &mut nw, // lpNumberOfCharsWritten
                    ptr::null_mut(), // lpReserved
                ) };
                if ret == FALSE { return Err(Error::last_os_error()); }

                // Taken from std
                let mut count = 0;
                for c in &wstr[..nw as usize] {
                    count += match c {
                        0x0000..=0x007F => 1,
                        0x0080..=0x07FF => 2,
                        0xDC00..=0xDFFF => 1, // Low surrogate. We already counted 3 bytes for the other.
                        _ => 3,
                    };
                }

                Ok(count)
            })
        } else {
            // Not a console, or UTF-8 console
            let mut nw: DWORD = 0;
            let ret = unsafe { WriteFile(
                handle, // hFile
                buf.as_ptr() as LPCVOID, // lpBuffer
                buf.len() as DWORD, // nNumberOfBytesToWrite
                &mut nw, // lpNumberOfBytesWritten
                ptr::null_mut(), // lpOverlapped
            ) };
            if ret == FALSE { return Err(Error::last_os_error()); }
            Ok(nw as usize)
        }
    }
}

fn with_static_utf16_buffer<T>(f: impl FnOnce(&mut Vec<u16>) -> T) -> T {
    static STATIC_BUFFER: Mutex<Vec<u16>> = Mutex::new(Vec::new());
    if let Some(mut guard) = STATIC_BUFFER.try_lock() {
        f(&mut *guard)
    } else {
        f(&mut Vec::new())
    }
}

#[allow(non_snake_case)]
fn is_console(hConsole: HANDLE) -> bool {
    let mut mode: DWORD = 0;
    let ret = unsafe { GetConsoleMode(hConsole, &mut mode) };
    ret != FALSE
}

#[allow(non_snake_case)]
fn is_win7() -> bool {
    // Althrough GetVersion misreports OS versions after 6.2, that doesn't affect our check
    let dwVersion = unsafe { GetVersion() };
    let dwMajorVersion = dwVersion & 0x00FF;
    let dwMinorVersion = (dwVersion & 0xFF00) >> 8;
    dwMajorVersion <= 6 && dwMinorVersion <= 1
}
