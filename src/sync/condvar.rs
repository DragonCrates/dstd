use core::sync::atomic::Ordering::*;

use super::sys::*;
use super::MutexGuard;

pub struct Condvar {
    word: FutexWord
}

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar {
            word: FutexWord::new(0)
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
