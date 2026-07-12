#![allow(non_camel_case_types, non_snake_case)]

use core::ptr;
use core::mem;

use crate::ffi::*;
use crate::io::{Error, CloseHandle};
use crate::prelude::Box;

use super::ThreadInit;

type LPTHREAD_START_ROUTINE = extern "C" fn(
    /* _In_ */ lpParameter: LPVOID,
) -> DWORD;

unsafe extern "C" {
    fn CreateThread(
        /* [in, optional] */ lpThreadAttributes: LPSECURITY_ATTRIBUTES,
        /* [in] */ dwStackSize: SIZE_T,
        /* [in] */ lpStartAddress: LPTHREAD_START_ROUTINE,
        /* [in, optional] */ lpParameter: LPVOID,
        /* [in] */ dwCreationFlags: DWORD,
        /* [out, optional]*/ lpThreadId: LPDWORD,
    ) -> HANDLE;
    fn WaitForSingleObject(
        /* [in] */ hHandle: HANDLE,
        /* [in] */ dwMilliseconds: DWORD,
    ) -> DWORD;
}

extern "C" fn start<F, T>(lpParameter: LPVOID) -> DWORD
where
    F: FnOnce() -> T
{
    let args = unsafe { Box::from_raw(lpParameter as *mut ThreadInit<F, T>) };
    let ret = (args.thread_main)();
    unsafe { *args.ret.get() = Some(ret); }
    0
}

pub fn spawn<F, T>(args: *mut ThreadInit<F, T>) -> JoinHandle
where
    F: FnOnce() -> T
{
    let mut thread: DWORD = 0;
    let ret = unsafe { CreateThread(
        ptr::null_mut(), // lpThreadAttributes
        0, // dwStackSize
        start::<F, T>, // lpStartAddress
        args as LPVOID, // lpParameter
        0, // dwCreationFlags
        &mut thread as LPDWORD, // lpThreadId
    ) };
    if ret.is_null() { panic!("CreateThread failed: {}", Error::last_os_error()); }
    JoinHandle(ret)
}

pub struct JoinHandle(HANDLE);

impl JoinHandle {
    pub fn join(self) {
        let ret = unsafe { WaitForSingleObject(self.0, DWORD::MAX) };
        if ret == 0xFFFFFFFF { panic!("WaitForSingleObject failed: {}", Error::last_os_error()); }
    }
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        let ret = unsafe { CloseHandle(self.0) };
        if ret == 0 { panic!("CloseHandle failed: {}", Error::last_os_error()) }
    }
}

// TODO: verify struct size
#[repr(C)]
struct SYSTEM_INFO {
    wProcessorArchitecture: WORD,
    wReserved: WORD,
    dwPageSize: DWORD,
    lpMinimumApplicationAddress: LPVOID,
    lpMaximumApplicationAddress: LPVOID,
    dwActiveProcessorMask: DWORD_PTR,
    dwNumberOfProcessors: DWORD,
    dwProcessorType: DWORD,
    dwAllocationGranularity: DWORD,
    wProcessorLevel: WORD,
    wProcessorRevision: WORD,
}
type LPSYSTEM_INFO = *mut SYSTEM_INFO;

unsafe extern "C" {
    fn GetSystemInfo(
        /* [out] */ lpSystemInfo: LPSYSTEM_INFO
    );
}

pub fn ncpu() -> usize {
    unsafe {
        let mut system_info: SYSTEM_INFO = mem::zeroed();
        GetSystemInfo(&mut system_info);
        system_info.dwNumberOfProcessors as usize
    }
}
