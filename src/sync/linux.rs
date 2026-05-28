#![allow(non_upper_case_globals)]

use core::ffi::c_long;
use core::sync::atomic::AtomicU32;

use crate::io::{Error, errno};

crate::cfg_if! {
    if #[cfg(target_arch = "x86")] {
        const SYS_futex: c_long = 240;
    } else if #[cfg(target_arch = "x86_64")] {
        const SYS_futex: c_long = 202;
    } else if #[cfg(target_arch = "arm")] {
        const SYS_futex: c_long = 240;
    } else if #[cfg(target_arch = "aarch64")] {
        const SYS_futex: c_long = 98;
    } else if #[cfg(target_arch = "riscv64")] {
        const SYS_futex: c_long = 98;
    }
}

unsafe extern "C" {
    fn syscall(__number: c_long, ...) -> c_long;
}

const FUTEX_WAIT: u32 = 0;
const FUTEX_WAKE: u32 = 1;
const FUTEX_PRIVATE_FLAG: u32 = 128;

/// Waits for a `futex_wake` operation to wake us.
///
/// Returns directly if the futex doesn't hold the expected value.
pub fn futex_wait(futex: &AtomicU32, expected: u32) {
    use core::ptr;

    let ret = unsafe {
        syscall(
            SYS_futex,
            futex as *const AtomicU32, // uaddr
            FUTEX_WAIT | FUTEX_PRIVATE_FLAG,
            expected, // val
            ptr::null::<u32>(), // timeout
        )
    };

    if ret == -1 {
        let err = Error::last_os_error();
        match err.raw_os_error().unwrap() {
            // EAGAIN - futex != expected
            // EINTR - interrupted by a signal
            errno::EAGAIN | errno::EINTR => {}
            // ETIMEDOUT - timed out (we didn't provide any)
            // EFAULT - invalid address
            // EINVAL - invalid timeout
            _ => panic!("futex_wait failed: {err}"),
        }
    }
}

/// Wakes up one thread that's blocked on `futex_wait` on this futex.
pub fn futex_wake(futex: &AtomicU32) {
    let ret = unsafe {
        syscall(
            SYS_futex,
            futex as *const AtomicU32,
            FUTEX_WAKE | FUTEX_PRIVATE_FLAG,
            1,
        )
    };

    if ret == -1 {
        panic!("futex_wake failed: {}", Error::last_os_error());
    }
}
