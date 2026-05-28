use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::fmt::{self, Debug};

use super::Futex;

pub struct Mutex<T> {
    futex: Futex,
    value: UnsafeCell<T>,
}

// std source code has an excellent explanation why are these impls sound
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Mutex<T> {
        Mutex {
            futex: Futex::new(),
            value: UnsafeCell::new(t),
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        self.futex.lock();
        MutexGuard::new(self)
    }

    pub fn try_lock<'a>(&'a self) -> Option<MutexGuard<'a, T>> {
        if self.futex.try_lock() {
            Some(MutexGuard::new(self))
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

#[must_use = "if unused the Mutex will immediately unlock"]
//#[must_not_suspend = "holding a MutexGuard across suspend \
//                      points can cause deadlocks, delays, \
//                      and cause Futures to not implement `Send`"]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    // make it non-Send so it is impossible to hold one across await points in async contexts
    __notsend: PhantomData<*mut T>,
}

impl<T> MutexGuard<'_, T> {
    // should only be called when mutex is locked
    fn new<'a>(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
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
