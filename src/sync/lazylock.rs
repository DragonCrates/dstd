use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;

use super::Once;

/// A value which is initialized on the first access
pub struct LazyLock<T, F> {
    init: Once,
    data: UnsafeCell<Data<T, F>>,
}

union Data<T, F> {
    f: ManuallyDrop<F>,
    value: ManuallyDrop<T>,
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Creates a new `LazyLock`
    pub const fn new(f: F) -> LazyLock<T, F> {
        LazyLock {
            init: Once::new(),
            data: UnsafeCell::new(Data { f: ManuallyDrop::new(f) }),
        }
    }

    /// Returns a reference to the contained value, running the provided function if it was not initialized
    pub fn get(&self) -> &T {
        self.init.call_once(|| {
            let data = unsafe { &mut *self.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });
        let data = unsafe { &*self.data.get() };
        unsafe { &data.value }
    }
}

// We never create a `&F` from a `&LazyLock<T, F>` so it is fine
// to not impl `Sync` for `F`.
unsafe impl<T: Sync + Send, F: Send> Sync for LazyLock<T, F> {}
// auto-derived `Send` impl is OK.
