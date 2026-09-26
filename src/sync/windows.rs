use core::sync::atomic::{AtomicU8, AtomicU32};

use crate::sys::windows::types::*;
use crate::io::Error;

pub trait Waitable {
    type Primitive;
    fn as_ptr(&self) -> *mut Self::Primitive;
}

impl Waitable for AtomicU8 {
    type Primitive = u8;
    fn as_ptr(&self) -> *mut u8 {
        self.as_ptr()
    }
}

impl Waitable for AtomicU32 {
    type Primitive = u32;
    fn as_ptr(&self) -> *mut u32 {
        self.as_ptr()
    }
}

pub type FutexWord = AtomicU32;
//pub type FutexPrimitive = u32;
pub type MiniFutex = AtomicU8;
pub type MiniPrimitive = u8;

unsafe extern "C" {
    /// Waits for the value at the specified address to change.
    fn WaitOnAddress(
        /* [in] */ Address: *mut /*volatile*/ VOID,
        /* [in] */ CompareAddress: PVOID,
        /* [in] */ AddressSize: SIZE_T,
        /* [in, optional] */ dwMilliseconds: DWORD,
    ) -> BOOL;
    /// Wakes one thread that is waiting for the value of an address to change.
    fn WakeByAddressSingle(/* [in] */ Address: PVOID);
    /// Wakes all threads that are waiting for the value of an address to change.
    fn WakeByAddressAll(/* [in] */ Address: PVOID);
}

pub fn futex_wait<T: Waitable>(futex: &T, expected: T::Primitive) {
    let ret = unsafe {
        WaitOnAddress(
            futex.as_ptr() as *mut VOID,    // Address
            &expected as *const _ as PVOID, // CompareAddress
            core::mem::size_of::<T>(),      // AddressSize
            u32::MAX,                       // dwMilliseconds
        )
    };

    if ret == 0 {
        panic!("WaitOnAddress failed: {}", Error::last_os_error());
    }
}

pub fn futex_wake<T: Waitable>(futex: &T) {
    unsafe {
        WakeByAddressSingle(futex.as_ptr() as PVOID);
    }
}

pub fn futex_wake_all<T: Waitable>(futex: &T) {
    unsafe {
        WakeByAddressAll(futex.as_ptr() as PVOID);
    }
}
