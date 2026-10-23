use core::ffi::c_void;

use crate::sys::windows::minwindef::*;
use crate::sys::windows::fibersapi::*;
use crate::io::Error;

pub type Key = DWORD;

pub fn tls_alloc(destructor: PFLS_CALLBACK_FUNCTION) -> Key {
    let ret = unsafe { FlsAlloc(destructor) };
    if ret == FLS_OUT_OF_INDEXES {
        panic!("FlsAlloc failed: {}", Error::last_os_error());
    }
    ret
}

pub fn tls_get_value(key: Key) -> *mut c_void {
    unsafe { FlsGetValue(key) }
}

pub fn tls_set_value(key: Key, value: *mut c_void) {
    let ret = unsafe { FlsSetValue(key, value) };
    if ret == 0 {
        panic!("FlsSetValue failed: {}", Error::last_os_error());
    }
}
