#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    mod linux;
    use linux::*;
}
#[cfg(target_os = "windows")]
crate::block! {
    mod windows;
    use windows::*;
}

mod futex;
use futex::Futex;

mod mutex;
pub use mutex::{Mutex, MutexGuard};

mod once;
pub use once::Once;

mod lazylock;
pub use lazylock::LazyLock;

extern crate alloc;
#[doc(no_inline)]
pub use alloc::sync::Arc;

// TODO condvar, SmallFutex, mpsc
// https://github.com/rust-lang/rust/blob/main/library/std/src/sys/sync/condvar/futex.rs
