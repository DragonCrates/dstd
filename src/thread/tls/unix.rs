use core::ffi::{c_uint, c_int, c_void};

pub type Key = c_uint;

unsafe extern "C" {
    /// Allocate a new TSD key
    fn pthread_key_create(key: *mut Key, destr_function: extern "C" fn(*mut c_void)) -> c_int;
    /// Changes the value associated with key
    fn pthread_setspecific(key: Key, pointer: *const c_void) -> c_int;
    /// Returns the value currently associated with key
    fn pthread_getspecific(key: Key) -> *mut c_void;
}

pub fn tls_alloc(destructor: extern "C" fn(*mut c_void)) -> Key {
    let mut key = 0;
    let ret = unsafe { pthread_key_create(&mut key, destructor) };
    assert!(ret == 0, "out of TSD keys");
    key
}

pub fn tls_get_value(key: Key) -> *mut c_void {
    unsafe { pthread_getspecific(key) }
}

pub fn tls_set_value(key: Key, value: *mut c_void) {
    let ret = unsafe { pthread_setspecific(key, value) };
    assert!(ret == 0, "invalid TSD key");
}
