use core::sync::atomic::Ordering::*;

use super::sys::*;

const UNLOCKED: MiniPrimitive = 0;
const LOCKED: MiniPrimitive = 1;
const CONTENDED: MiniPrimitive = 2;

pub struct RawMutex {
    word: MiniFutex
}

impl RawMutex {
    /// Constructs a new mutex
    pub const fn new() -> RawMutex {
        RawMutex {
            word: MiniFutex::new(UNLOCKED)
        }
    }

    /// Locks the mutex
    pub fn lock(&self) {
        if !self.try_lock() {
            self.lock_contended();
        }
    }

    /// Lock is not free, should wait
    #[cold]
    fn lock_contended(&self) {
        loop {
            if self.word.swap(CONTENDED, Acquire) == UNLOCKED {
                // We've just locked it now
                return;
            }
            futex_wait(&self.word, CONTENDED);
        }
    }

    /// Locks without blocking. true = success, false = fail
    pub fn try_lock(&self) -> bool {
        self.word.compare_exchange(UNLOCKED, LOCKED, Acquire, Relaxed).is_ok()
    }

    /// Unlocks the mutex
    pub fn unlock(&self) {
        let state = self.word.swap(UNLOCKED, Release);
        match state {
            // Never happens
            UNLOCKED => panic!("attempt to unlock an unlocked mutex"),
            // No need to syscall if there were no waiters
            LOCKED => {},
            // Has waiters, do wake
            CONTENDED => futex_wake(&self.word),
            // Unreachable
            _ => unreachable!("invalid mutex state"),
        }
    }
}
