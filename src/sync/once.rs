use core::sync::atomic::Ordering::*;

use super::sys::*;

/// A low-level synchronization primitive for one-time global execution
pub struct Once {
    state: MiniFutex
}

const UNINIT: MiniPrimitive = 0;
const LOCKED: MiniPrimitive = 1;
const CONTENDED: MiniPrimitive = 2;
const COMPLETE: MiniPrimitive = 3;

impl Once {
    /// Creates a new Once value
    pub const fn new() -> Once {
        Once {
            state: MiniFutex::new(UNINIT)
        }
    }

    pub(crate) const fn new_complete() -> Once {
        Once {
            state: MiniFutex::new(COMPLETE)
        }
    }

    /// Performs the initialization only once
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        let state = self.state.load(Acquire);
        if state != COMPLETE {
            self.call_once_slow(state, f);
        }
    }

    #[cold]
    fn call_once_slow<F: FnOnce()>(&self, mut state: MiniPrimitive, f: F) {
        loop {
            if state == UNINIT {
                match self.state.compare_exchange(UNINIT, LOCKED, Acquire, Acquire) {
                    Ok(_) => {
                        // Lock acquired, perform init
                        f();
                        // Done, now unlock
                        let old = self.state.swap(COMPLETE, Release);
                        if old == CONTENDED {
                            futex_wake_all(&self.state);
                        }
                        return;
                    }
                    Err(actual) => state = actual
                }
            } else if state == LOCKED {
                // It is locked, register contention
                match self.state.compare_exchange_weak(LOCKED, CONTENDED, Relaxed, Acquire) {
                    Ok(_) | Err(CONTENDED) => {
                        // Now, wait
                        futex_wait(&self.state, CONTENDED);
                        // Woke up, reload the state
                        state = self.state.load(Acquire);
                    }
                    Err(actual) => state = actual
                }
            } else if state == CONTENDED {
                // Already contended, just wait
                futex_wait(&self.state, CONTENDED);
                state = self.state.load(Acquire);
            } else if state == COMPLETE {
                // We are done
                return;
            } else {
                #[cfg(debug_assertions)]
                panic!("unreachable Once state: {state}");
            }
        }
    }

    pub(crate) fn call_once_mut<F: FnOnce()>(&mut self, f: F) {
        let state = self.state.get_mut();
        if *state == UNINIT {
            f();
            *state = COMPLETE;
        } else if *state == COMPLETE {
            // Do nothing
        } else {
            #[cfg(debug_assertions)]
            panic!("unreachable Once state: {state}");
        }
    }

    /// Returns `true` if initializarion is completed
    pub fn is_completed(&self) -> bool {
        self.state.load(Acquire) == COMPLETE
    }

    pub(crate) fn is_completed_mut(&mut self) -> bool {
        *self.state.get_mut() == COMPLETE
    }
}

impl Default for Once {
    fn default() -> Once {
        Once::new()
    }
}
