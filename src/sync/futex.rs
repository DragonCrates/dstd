use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::{Relaxed, Acquire, Release};

use super::{futex_wait, futex_wake};

const UNLOCKED: u32 = 0;
const LOCKED: u32 = 1;
const CONTENDED: u32 = 2;

pub struct Futex(AtomicU32);

impl Futex {
    /// Constructs a new mutex
    pub const fn new() -> Futex {
        Futex(AtomicU32::new(UNLOCKED))
    }

    /// Locks the mutex
    pub fn lock(&self) {
        if !self.try_lock() {
            self.lock_contended();
        }
    }

    /// Lock is not free, should wait
    #[cold]
    pub fn lock_contended(&self) {
        loop {
            if self.0.swap(CONTENDED, Acquire) == UNLOCKED {
                // We've just locked it now
                return;
            }
            futex_wait(&self.0, CONTENDED);
        }
    }

    /// Locks without blocking. true = success, false = fail
    pub fn try_lock(&self) -> bool {
        self.0.compare_exchange(UNLOCKED, LOCKED, Acquire, Relaxed).is_ok()
    }

    /// Unlocks the mutex
    pub fn unlock(&self) {
        let state = self.0.swap(UNLOCKED, Release);
        match state {
            // Never happens
            UNLOCKED => panic!("attempt to unlock an unlocked mutex"),
            // No need to syscall if there were no waiters
            LOCKED => return,
            // Has waiters, do wake
            CONTENDED => futex_wake(&self.0),
            // Unreachable, do nothing
            _ => {},
        }
    }
}
