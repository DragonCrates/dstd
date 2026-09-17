//! Synchronization primitives

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

mod oncelock;
pub use oncelock::OnceLock;

mod lazylock;
pub use lazylock::LazyLock;

extern crate alloc;
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};

// TODO condvar, SmallFutex, mpsc, Barrier, Semaphore
// https://github.com/rust-lang/rust/blob/main/library/std/src/sys/sync/condvar/futex.rs
