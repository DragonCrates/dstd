//! Synchronization primitives

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    mod linux;
    use linux as sys;
}
#[cfg(target_os = "windows")]
crate::block! {
    mod windows;
    use windows as sys;
}

mod raw_mutex;
use raw_mutex::RawMutex;

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

mod condvar;
pub use condvar::Condvar;

// TODO mpsc, Barrier, Semaphore
