use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

use super::Once;

/// A synchronization primitive that can be written only once
///
/// `OnceLock` is a less convenient version of [`LazyLock`], because it doesn't implement [`Deref`].
/// However, it is more flexible, because you can supply your own initializer
/// # Example
/// ```
/// use dstd::sync::OnceLock;
///
/// fn get_value() -> &'static str {
///     static VALUE: OnceLock<String> = OnceLock::new();
///     // You can reference local variables from `OnceLock`s initializer
///     let meaning = "42".to_string();
///     VALUE.get_or_init(|| meaning.clone()).as_str()
/// }
///
/// println!("{}", get_value());
/// ```
/// [`LazyLock`]: super::LazyLock
/// [`Deref`]: core::ops::Deref
pub struct OnceLock<T> {
    init: Once,
    data: UnsafeCell<MaybeUninit<T>>,
    /// <https://github.com/rust-lang/rust/blob/c33d8f3b5a50b56466998e8c5ed8a077d2caed84/library/std/src/sync/once_lock.rs#L114>
    /// ```compile_fail,E0597
    /// use dstd::sync::OnceLock;
    ///
    /// struct A<'a>(&'a str);
    ///
    /// impl<'a> Drop for A<'a> {
    ///     fn drop(&mut self) {}
    /// }
    ///
    /// let cell = OnceLock::new();
    /// {
    ///     let s = String::new();
    ///     let _ = cell.set(A(&s));
    /// }
    /// ```
    marker: PhantomData<T>,
}

impl<T> OnceLock<T> {
    /// Constructs a new uninitialized `OnceLock`
    #[inline]
    #[must_use]
    pub const fn new() -> OnceLock<T> {
        OnceLock {
            init: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            marker: PhantomData,
        }
    }

    /// Get the reference to the underlying value, if it was initialized
    pub fn get(&self) -> Option<&T> {
        if self.init.is_completed() {
            let data = unsafe { &*self.data.get() };
            Some(unsafe { data.assume_init_ref() })
        } else {
            None
        }
    }

    /// Get the mutable reference to the underlying value
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.init.is_completed_mut() {
            let data = unsafe { &mut *self.data.get() };
            Some(unsafe { data.assume_init_mut() })
        } else {
            None
        }
    }

    /// Initializes the contents of the cell to `value`
    ///
    /// Returns `Err` if value is already initialized
    pub fn set(&self, value: T) -> Result<(), T> {
        let mut value = Some(value);
        self.init.call_once(|| {
            unsafe {
                let data = &mut *self.data.get();
                data.write(value.take().unwrap());
            }
        });

        match value {
            Some(value) => Err(value),
            None => Ok(()),
        }
    }

    /// Returns the contents of cell, initializing it if necessary
    ///
    /// If multiple threads are trying to run the initializer concurrently, only one will run, and all others would block
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        self.init.call_once(|| {
            unsafe {
                let data = &mut *self.data.get();
                data.write(f());
            }
        });
        let data = unsafe { &*self.data.get() };
        unsafe { data.assume_init_ref() }
    }

    /// Consumes the `OnceLock`, returning the wrapped value
    #[inline]
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    /// Takes the value out of this `OnceLock`, replacing it with uninitialized state
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        if self.init.is_completed_mut() {
            self.init = Once::new();
            let data = unsafe { &mut *self.data.get() };
            Some(unsafe { data.assume_init_read() })
        } else {
            None
        }
    }
}

impl<T> Drop for OnceLock<T> {
    fn drop(&mut self) {
        self.take();
    }
}

unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}

impl<T> Default for OnceLock<T> {
    /// Creates a new uninitialized cell.
    ///
    /// # Example
    ///
    /// ```
    /// use dstd::sync::OnceLock;
    ///
    /// fn main() {
    ///     assert_eq!(OnceLock::<()>::new(), OnceLock::default());
    /// }
    /// ```
    fn default() -> OnceLock<T> {
        OnceLock::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for OnceLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("OnceLock");
        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

impl<T> Clone for OnceLock<T> {
    fn clone(&self) -> OnceLock<T> {
        self.get().cloned().map_or_default(OnceLock::from)
    }
}

impl<T> From<T> for OnceLock<T> {
    fn from(value: T) -> OnceLock<T> {
        OnceLock {
            init: Once::new_complete(),
            data: UnsafeCell::new(MaybeUninit::new(value)),
            marker: PhantomData,
        }
    }
}

impl<T: PartialEq> PartialEq for OnceLock<T> {
    #[inline]
    fn eq(&self, other: &OnceLock<T>) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for OnceLock<T> {}
