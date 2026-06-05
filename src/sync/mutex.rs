use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::fmt::{self, Debug};

use super::Futex;

/// Mutex primitive, used to protect shared data
pub struct Mutex<T> {
    futex: Futex,
    value: UnsafeCell<T>,
}

/// `Send` means that it is allowed to pass `T` between threads by value. `Mutex` is `Send` only when the contained type is `Send`
unsafe impl<T: Send> Send for Mutex<T> {}
/// `Mutex` allows access to value from any thread. Therefore, for `Mutex` to be `Sync`, the value should be `Send`.
///
/// It is not necessary to be `Sync`, because `Mutex` only allows one access at a time (only `&mut T`, not `&T`)
///
/// Reference: <https://github.com/rust-lang/rust/blob/9e293ae9f8abecb0be5105787d181518c9012a19/library/std/src/sync/poison/mutex.rs#L240>
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new `Mutex`
    pub const fn new(t: T) -> Mutex<T> {
        Mutex {
            futex: Futex::new(),
            value: UnsafeCell::new(t),
        }
    }

    /// Locks a mutex
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.futex.lock();
        unsafe { MutexGuard::new(self) }
    }

    /// Tries to acquire a lock without waiting
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.futex.try_lock() {
            unsafe { Some(MutexGuard::new(self)) }
        } else {
            None
        }
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(T::default())
    }
}

impl<T: Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Some(v) => d.field("value", &*v),
            None => d.field("value", &"<locked>"),
        };
        d.finish()
    }
}

/// Mutex guard, that is automatically released on drop
///
/// To access the contained value, use [`Deref`] or [`DerefMut`] traits:
/// `*value`
#[must_use = "if unused the Mutex will immediately unlock"]
//#[must_not_suspend = "holding a MutexGuard across suspend \
//                      points can cause deadlocks, delays, \
//                      and cause Futures to not implement `Send`"]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    // make it non-Send so it is impossible to hold one across await points in async contexts
    __notsend: PhantomData<*mut T>,
}

/// `MutexGuard` is `Sync` when `T` is `Sync` (because `&MutexGuard<T>` is equivalent to `&T`)
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> MutexGuard<'_, T> {
    // safety: should only be called when mutex is locked
    unsafe fn new(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        MutexGuard { mutex, __notsend: PhantomData }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.futex.unlock();
    }
}
