use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Relaxed, Acquire, Release};

use super::Futex;

// TODO: make it use only one field
/// A low-level synchronization primitive for one-time global execution
pub struct Once {
    inited: AtomicBool,
    futex: Futex,
}

impl Once {
    /// Creates a new Once value
    pub const fn new() -> Once {
        Once {
            inited: AtomicBool::new(false),
            futex: Futex::new(),
        }
    }

    pub(crate) const fn new_complete() -> Once {
        Once {
            inited: AtomicBool::new(true),
            futex: Futex::new(),
        }
    }

    /// Performs the initialization only once
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if !self.inited.load(Acquire) {
            self.futex.lock();
            if !self.inited.load(Relaxed) {
                f();
                self.inited.store(true, Release);
            }
            self.futex.unlock();
        }
    }

    pub(crate) fn call_once_mut<F: FnOnce()>(&mut self, f: F) {
        let inited = self.inited.get_mut();
        if !*inited {
            f();
            *inited = true;
        }
    }

    /// Returns `true` if initializarion is completed
    pub fn is_completed(&self) -> bool {
        self.inited.load(Acquire)
    }

    pub(crate) fn is_completed_mut(&mut self) -> bool {
        *self.inited.get_mut()
    }
}

impl Default for Once {
    fn default() -> Once {
        Once::new()
    }
}
