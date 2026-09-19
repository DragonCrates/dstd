use core::ffi::c_void;

use crate::sys::windows::types::{DWORD, PVOID, BOOL};
use crate::io::Error;

pub type Key = DWORD;

#[allow(nonstandard_style)]
type PFLS_CALLBACK_FUNCTION = extern "C" fn(
    /* [in] */ lpFlsData: PVOID,
);
unsafe extern "C" {
    /// Allocates a fiber local storage (FLS) index
    fn FlsAlloc(
        /* [in] */ lpCallback: PFLS_CALLBACK_FUNCTION,
    ) -> DWORD;
    /// Retrieves the value in the calling fiber's FLS slot for the specified FLS index
    fn FlsGetValue(
        /* [in] */ dwFlsIndex: DWORD
    ) -> PVOID;
    /// Stores a value in the calling fiber's FLS slot for the specified FLS index
    fn FlsSetValue(
        /* [in] */ dwFlsIndex: DWORD,
        /* [in, optional] */ lpFlsData: PVOID,
    ) -> BOOL;
}

const FLS_OUT_OF_INDEXES: DWORD = 0xffffffff;

pub fn tls_alloc(destructor: extern "C" fn(*mut c_void)) -> Key {
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
