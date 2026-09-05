use core::cell::UnsafeCell;
use core::fmt;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::Once;

/// A value which is initialized on the first access
pub struct LazyLock<T, F = fn() -> T> {
    init: Once,
    data: UnsafeCell<Data<T, F>>,
}

union Data<T, F> {
    f: ManuallyDrop<F>,
    value: ManuallyDrop<T>,
}

impl<T, F> LazyLock<T, F>
where
    F: FnOnce() -> T
{
    /// Creates a new `LazyLock`
    #[inline]
    #[must_use]
    pub const fn new(f: F) -> LazyLock<T, F> {
        LazyLock {
            init: Once::new(),
            data: UnsafeCell::new(Data { f: ManuallyDrop::new(f) }),
        }
    }

    /// Forces the initialization of this lazy value, and returns a reference to result
    ///
    /// This method will block if it is being initialized by an another thread
    pub fn force(&self) -> &T {
        self.init.call_once(|| {
            let data = unsafe { &mut *self.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });
        let data = unsafe { &*self.data.get() };
        unsafe { &data.value }
    }

    /// Forces initialization of this lazy value, and returns a mutable reference to result
    pub fn force_mut(&mut self) -> &mut T {
        self.init.call_once_mut(|| {
            let data = unsafe { &mut *self.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });
        unsafe { &mut self.data.get_mut().value }
    }
}

impl<T, F> LazyLock<T, F> {
    /// Returns a reference to the value if it was initialized, and `None` otherwise
    pub fn get(&self) -> Option<&T> {
        if self.init.is_completed() {
            let data = unsafe { &*self.data.get() };
            Some(unsafe { &data.value })
        } else {
            None
        }
    }
}

impl<T, F> Drop for LazyLock<T, F> {
    fn drop(&mut self) {
        if self.init.is_completed_mut() {
            unsafe { ManuallyDrop::drop(&mut self.data.get_mut().value); }
        } else {
            unsafe { ManuallyDrop::drop(&mut self.data.get_mut().f); }
        }
    }
}

impl<T, F> Deref for LazyLock<T, F>
where
    F: FnOnce() -> T
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.force()
    }
}

impl<T, F> DerefMut for LazyLock<T, F>
where
    F: FnOnce() -> T
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.force_mut()
    }
}

impl<T: Default> Default for LazyLock<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    #[inline]
    fn default() -> LazyLock<T> {
        LazyLock::new(T::default)
    }
}

impl<T: fmt::Debug, F> fmt::Debug for LazyLock<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("LazyLock");
        match LazyLock::get(self) {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

impl<T, F> From<T> for LazyLock<T, F> {
    /// Constructs a `LazyLock` that starts already initialized with the provided value
    #[inline]
    fn from(value: T) -> Self {
        LazyLock {
            init: Once::new_complete(),
            data: UnsafeCell::new(Data { value: ManuallyDrop::new(value) }),
        }
    }
}

// We never create a `&F` from a `&LazyLock<T, F>` so it is fine
// to not impl `Sync` for `F`.
unsafe impl<T: Send + Sync, F: Send> Sync for LazyLock<T, F> {}
// auto-derived `Send` impl is OK.
