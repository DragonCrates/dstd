use core::ptr;

use crate::sys::windows::{CreateFileW, ReadFile, WriteFile, SetFilePointerEx};
use crate::sys::windows::types::*;

use super::OpenOptions;
use crate::io::{SeekFrom, Result, Error};
use crate::os_str::OsStr;

pub type Handle = HANDLE;

const FILE_READ_DATA: DWORD = 0x0001;
const FILE_WRITE_DATA: DWORD = 0x0002;
const FILE_APPEND_DATA: DWORD = 0x0004;

fn access(opts: &OpenOptions) -> DWORD {
    let mut flag = 0;
    if opts.read { flag |= FILE_READ_DATA; }
    // Append should be without write
    if opts.append {
        flag |= FILE_APPEND_DATA;
    } else if opts.write {
        flag |= FILE_WRITE_DATA;
    }
    flag
}

const CREATE_NEW: DWORD = 1;
const CREATE_ALWAYS: DWORD = 2;
const OPEN_EXISTING: DWORD = 3;
const OPEN_ALWAYS: DWORD = 4;
const TRUNCATE_EXISTING: DWORD = 5;

fn disposition(opts: &OpenOptions) -> DWORD {
    if opts.create {
        if opts.excl {
            // create = true, excl = true
            CREATE_NEW
        } else if opts.truncate {
            // create = true, excl = false, truncate = true
            CREATE_ALWAYS
        } else {
            // create = true, excl = false, truncate = false
            OPEN_ALWAYS
        }
    } else if opts.truncate {
        // create = false, truncate = true
        TRUNCATE_EXISTING
    } else {
        // create = false, truncate = false
        OPEN_EXISTING
    }
}

const FILE_SHARE_READ: DWORD = 0x00000001;
const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub fn open(name: &OsStr, opts: &OpenOptions) -> Result<HANDLE> {
    let access = access(opts);
    let disposition = disposition(opts);
    let ret = unsafe { CreateFileW(
        name.as_ptr(), // lpFileName
        access, // dwDesiredAccess
        FILE_SHARE_READ | FILE_SHARE_WRITE, // dwShareMode
        ptr::null_mut(), // lpSecurityAttributes
        disposition, // dwCreationDisposition
        0, // dwFlagsAndAttributes
        ptr::null_mut(), // hTemplateFile
    ) };
    if ret == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }
    Ok(ret)
}

pub fn read(handle: HANDLE, buf: &mut [u8]) -> Result<usize> {
    let mut nr: DWORD = 0;
    let ret = unsafe { ReadFile(
        handle, // hFile
        buf.as_mut_ptr() as LPVOID, // lpBuffer
        buf.len() as DWORD, // nNumberOfBytesToRead
        &mut nr, // lpNumberOfBytesRead
        ptr::null_mut(), // lpOverlapped
    ) };
    if ret == 0 { return Err(Error::last_os_error()); }
    Ok(nr as usize)
}

pub fn write(handle: HANDLE, buf: &[u8]) -> Result<usize> {
    let mut nw: DWORD = 0;
    let ret = unsafe { WriteFile(
        handle, // hFile
        buf.as_ptr() as LPCVOID, // lpBuffer
        buf.len() as DWORD, // nNumberOfBytesToWrite
        &mut nw, // lpNumberOfBytesWritten
        ptr::null_mut(), // lpOverlapped
    ) };
    if ret == 0 { return Err(Error::last_os_error()); }
    Ok(nw as usize)
}

pub fn seek(handle: HANDLE, pos: SeekFrom) -> Result<u64> {
    let (dist, method) = pos.to_flags();
    let mut new_file_pointer: i64 = 0;
    let ret = unsafe { SetFilePointerEx(
        handle, // hFile
        dist as i64, // liDistanceToMove
        &mut new_file_pointer, // lpNewFilePointer
        method as DWORD, // dwMoveMethod
    ) };
    if ret == 0 { return Err(Error::last_os_error()); }
    Ok(new_file_pointer as u64)
}
