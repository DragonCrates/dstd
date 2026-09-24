use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::*;

use super::{futex_wait, futex_wake, futex_wake_all, MutexGuard};

pub struct Condvar {
    word: AtomicU32
}

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar {
            word: AtomicU32::new(0)
        }
    }

    pub fn notify_one(&self) {
        self.word.fetch_add(1, Relaxed);
        futex_wake(&self.word);
    }

    pub fn notify_all(&self) {
        self.word.fetch_add(1, Relaxed);
        futex_wake_all(&self.word);
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let value = self.word.load(Relaxed);
        let mtx = guard.into_mutex();
        futex_wait(&self.word, value);
        mtx.lock()
    }
}
