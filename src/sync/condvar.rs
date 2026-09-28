use core::sync::atomic::Ordering::*;
use core::sync::atomic::AtomicU32;

use super::sys::*;
use super::MutexGuard;

pub struct Condvar {
    word: FutexWord,
    waiters: AtomicU32,
}

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar {
            word: FutexWord::new(0),
            waiters: AtomicU32::new(0),
        }
    }

    pub fn notify_one(&self) {
        if self.waiters.load(Relaxed) == 0 { return; }
        self.word.fetch_add(1, Relaxed);
        futex_wake(&self.word);
    }

    pub fn notify_all(&self) {
        if self.waiters.load(Relaxed) == 0 { return; }
        self.word.fetch_add(1, Relaxed);
        futex_wake_all(&self.word);
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let value = self.word.load(Relaxed);
        let mtx = guard.into_mutex();
        self.waiters.fetch_add(1, Relaxed);
        futex_wait(&self.word, value);
        self.waiters.fetch_sub(1, Relaxed);
        mtx.lock()
    }
}
