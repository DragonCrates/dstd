use core::cell::Cell;
use core::ffi::c_void;
use core::marker::PhantomData;

extern crate alloc;
use alloc::boxed::Box;

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

// TODO: LocalKey should only use the system allocator
pub struct LocalKey<T: 'static> {
    key: OnceLock<sys::Key>,
    value: PhantomData<T>,
    initializer: fn() -> T,
}

extern "C" fn destroy<T>(value: *mut c_void) {
    // No null pointer check because destructors only run for non null values
    let val = unsafe { Box::from_raw(value as *mut T) };
    drop(val);
}

impl<T> LocalKey<T> {
    /// Not a public api! Exposed only for the `thread_local!` macro
    #[doc(hidden)]
    pub const fn __new(f: fn() -> T) -> LocalKey<T> {
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

    pub fn with<R>(&'static self, f: impl FnOnce(&T) -> R) -> R {
        let mut value_ptr = self.value();
        if value_ptr.is_null() {
            // Initialize value
            let key = self.key();
            let value = Box::new((self.initializer)());
            let value = Box::into_raw(value);
            sys::tls_set_value(key, value as *mut c_void);
            value_ptr = value;
        }

        let value_ref = unsafe { &*value_ptr };
        f(value_ref)
    }
}

// TODO: other Cell and RefCell methods
impl<T: Copy> LocalKey<Cell<T>> {
    pub fn set(&'static self, value: T) {
        let value_ptr = self.value();
        if value_ptr.is_null() {
            // Initialize
            let key = self.key();
            let value = Box::new(value);
            let value = Box::into_raw(value);
            sys::tls_set_value(key, value as *mut c_void);
            return;
        }

        // Already initialized
        self.with(|cell| cell.set(value));
    }

    pub fn get(&'static self) -> T {
        self.with(|cell| cell.get())
    }

    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }

    pub fn update(&'static self, f: impl FnOnce(T) -> T) {
        self.set(f(self.get()));
    }
}

// TODO: per comment in Bionic,
/*
 * [pthread_key_create(3)](https://man7.org/linux/man-pages/man3/pthread_key_create.3p.html)
 * creates a key for thread-specific data.
 *
 * There is a limit of `PTHREAD_KEYS_MAX` keys per process, but most callers
 * should just use the C or C++ `thread_local` storage specifier anyway. When
 * targeting new enough OS versions, the compiler will automatically use
 * ELF TLS; when targeting old OS versions the emutls implementation will
 * multiplex pthread keys behind the scenes, using one per library rather than
 * one per thread-local variable. If you are implementing the runtime for a
 * different language, you should consider similar implementation choices and
 * avoid a direct one-to-one mapping from thread locals to pthread keys.
 *
 * Returns 0 on success and returns an error number on failure.
 */
// int pthread_key_create(pthread_key_t* _Nonnull __key_ptr, void (* _Nullable __key_destructor)(void* _Nullable));
// We need just a TypeMap

// TODO:
// When a rust cdylib module is unloaded, destructors will point to unmapped memory
// To avoid this, we should destroy all thread locals on module unload (via __cxa_atexit on linux/mac and atexit on windows)
// This is not implemented yet

#[macro_export]
macro_rules! thread_local {
    ($($(#[$m:meta])* $v:vis static $n:ident: $t:ty = $i:expr);* $(;)?) => {
        $(
            $(#[$m])* $v static $n: $crate::thread::LocalKey<$t> = $crate::thread::LocalKey::__new(|| $i);
        )*
    }
}

// LocalKey never shares references across threads, so it implements Sync even when T doesn't
unsafe impl<T> Sync for LocalKey<T> {}
