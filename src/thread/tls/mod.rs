use core::cell::{Cell, RefCell};
use core::ffi::c_void;
use core::marker::PhantomData;

extern crate alloc;
use alloc::alloc::{Layout, GlobalAlloc};

use crate::alloc::System;
use crate::sync::OnceLock;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

/// A thread local storage (TLS) key, instantiated with the [`crate::thread_local`] macro
///
/// Method [`LocalKey::with`] yields a shared reference to the contained value. Use [`Cell`] or [`RefCell`] to obtain an exclusive reference
/// # Implementation notes
/// This is implemented using `FlsGetValue` on Windows and `pthread_getspecific` on unix targets. Native TLS is not supported, and keys are not multiplexed
///
/// That also means that you can't create more keys than platform allows, and some platforms have very small amount of available keys (`PTHREAD_KEYS_MAX` is 128 on Android)
/// <details>
/// <summary>If (for any reason) you need true native TLS, you can add the following C++ shim to your project:</summary>
///
/// ```c++
#[doc = include_str!("../../../examples/examples/tls.cpp")]
/// ```
/// </details>
pub struct LocalKey<T: 'static> {
    key: OnceLock<sys::Key>,
    value: PhantomData<T>,
    initializer: fn() -> T,
}

extern "C" fn destroy<T>(value: *mut c_void) {
    // No null pointer check because destructors only run for non null values
    unsafe {
        value.drop_in_place();
        System.dealloc(value as *mut u8, Layout::new::<T>());
    }
}

impl<T> LocalKey<T> {
    /// Not a public api! Exposed only for the `thread_local!` macro
    #[doc(hidden)]
    pub const fn __dstd_macro_api_new(f: fn() -> T) -> LocalKey<T> {
        LocalKey {
            key: OnceLock::new(),
            value: PhantomData,
            initializer: f,
        }
    }

    fn key(&'static self) -> sys::Key {
        *self.key.get_or_init(|| {
            sys::tls_alloc(destroy::<T>)
        })
    }

    fn value(&'static self) -> *mut T {
        let key = self.key();
        sys::tls_get_value(key) as *mut T
    }

    fn init(&'static self, value: T) -> *mut T {
        let value_ptr;
        let key = self.key();
        unsafe {
            value_ptr = System.alloc(Layout::for_value(&value)) as *mut T;
            value_ptr.write(value);
        }
        sys::tls_set_value(key, value_ptr as *mut c_void);
        value_ptr
    }

    /// Acquires a reference to the stored value, initializing it if necessary
    /// # Example
    /// ```
    /// use core::cell::Cell;
    /// use dstd::thread_local;
    /// thread_local! {
    ///     static GREETING: String = "Hello dstd".to_string();
    /// }
    /// GREETING.with(|s| {
    ///     assert_eq!(s, "Hello dstd");
    /// });
    /// ```
    pub fn with<R>(&'static self, f: impl FnOnce(&T) -> R) -> R {
        let mut value_ptr = self.value();
        if value_ptr.is_null() {
            // Initialize
            let value = (self.initializer)();
            value_ptr = self.init(value);
        }

        let value_ref = unsafe { &*value_ptr };
        f(value_ref)
    }
}

impl<T> LocalKey<Cell<T>> {
    /// Sets the contained value, without running the initializer
    pub fn set(&'static self, value: T) {
        let value_ptr = self.value();
        if value_ptr.is_null() {
            // Initialize
            self.init(Cell::new(value));
        } else {
            // Already initialized
            self.with(|cell| cell.set(value));
        }
    }

    /// Returns a copy of the contained value
    pub fn get(&'static self) -> T
    where
        T: Copy
    {
        self.with(|cell| cell.get())
    }

    /// Takes the contained value, leaving `Default::default()` in its place
    pub fn take(&'static self) -> T
    where
        T: Default
    {
        self.with(|cell| cell.take())
    }

    /// Replaces the contained value, returning the old value
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }

    /// Updates the contained value using a function
    pub fn update(&'static self, f: impl FnOnce(T) -> T)
    where
        T: Copy
    {
        self.set(f(self.get()));
    }
}

impl<T> LocalKey<RefCell<T>> {
    /// Acquires a reference to the contained value
    pub fn with_borrow<R>(&'static self, f: impl FnOnce(&T) -> R) -> R {
        self.with(|cell| f(&cell.borrow()))
    }

    /// Acquires a mutable reference to the contained value
    pub fn with_borrow_mut<R>(&'static self, f: impl FnOnce(&mut T) -> R) -> R {
        self.with(|cell| f(&mut cell.borrow_mut()))
    }

    /// Sets the contained value, without running the initializer
    pub fn set(&'static self, value: T) {
        let value_ptr = self.value();
        if value_ptr.is_null() {
            // Initialize
            self.init(RefCell::new(value));
        } else {
            // Already initialized
            self.with_borrow_mut(|v| *v = value);
        }
    }

    /// Takes the contained value, leaving `Default::default()` in its place
    pub fn take(&'static self) -> T
    where
        T: Default
    {
        self.with(|cell| cell.take())
    }

    /// Replaces the contained value, returning the old value
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}

// TODO:
// When a rust cdylib module is unloaded, destructors will point to unmapped memory
// To avoid this, we should destroy all thread locals on module unload (via __cxa_atexit on linux/mac and atexit on windows)
// This is not implemented yet
// TODO:
// We can multiplex keys into a Vec

#[macro_export]
macro_rules! thread_local {
    ($($(#[$m:meta])* $v:vis static $n:ident: $t:ty = $i:expr);* $(;)?) => {
        $(
            $(#[$m])* $v static $n: $crate::thread::LocalKey<$t> = $crate::thread::LocalKey::__dstd_macro_api_new(|| $i);
        )*
    }
}

// LocalKey never shares references across threads, so it implements Sync even when T doesn't
unsafe impl<T> Sync for LocalKey<T> {}
