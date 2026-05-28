use core::sync::atomic::AtomicU32;

use crate::ffi::*;
use crate::io::Error;

unsafe extern "C" {
    /// Waits for the value at the specified address to change.
    fn WaitOnAddress(
        /* [in] */ Address: *mut VOID, // should be volatile
        /* [in] */ CompareAddress: *const VOID, // was PVOID
        /* [in] */ AddressSize: SIZE_T,
        /* [in, optional] */ dwMilliseconds: DWORD,
    ) -> BOOL;
    /// Wakes one thread that is waiting for the value of an address to change.
    fn WakeByAddressSingle(/* [in] */ Address: PVOID);
}

pub fn futex_wait(futex: &AtomicU32, expected: u32) {
    let ret = unsafe {
        WaitOnAddress(
            futex.as_ptr() as *mut VOID,            // Address
            &expected as *const u32 as *const VOID, // CompareAddress
            core::mem::size_of::<u32>(),            // AddressSize
            u32::MAX,                               // dwMilliseconds
        )
    };

    if ret == 0 {
        panic!("WaitOnAddress failed: {}", Error::last_os_error());
    }
}

pub fn futex_wake(futex: &AtomicU32) {
    unsafe {
        WakeByAddressSingle(futex.as_ptr() as PVOID);
    }
}
