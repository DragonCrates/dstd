use core::ptr;
use core::ffi::c_int;

extern crate alloc;
use alloc::vec;

use crate::io::{Result, Error, Read, Write};
use crate::sys::windows::types::*;
use crate::sys::windows::winbase::*;
use crate::sys::windows::WriteFile;
use crate::sys::windows::consoleapi::{GetConsoleMode, WriteConsoleW};
use crate::sys::windows::stringapiset::MultiByteToWideChar;
use crate::sys::windows::processenv::GetStdHandle;

pub struct RawStdio(DWORD);

pub fn raw_stdin() -> RawStdio {
    RawStdio(STD_INPUT_HANDLE)
}

pub fn raw_stdout() -> RawStdio {
    RawStdio(STD_OUTPUT_HANDLE)
}

pub fn raw_stderr() -> RawStdio {
    RawStdio(STD_ERROR_HANDLE)
}

impl Read for RawStdio {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        // TODO
        Ok(0)
    }
}

impl Write for RawStdio {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
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
            // Not a console
            let mut nw: DWORD = 0;
            let ret = unsafe { WriteFile(
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
