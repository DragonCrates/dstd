#![allow(non_camel_case_types)]

use core::ptr;

use crate::ffi::*;
use crate::io::Error;
use crate::prelude::Box;

use super::ThreadInit;

crate::cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "android"))] {
        pub type pthread_t = c_ulong;

        #[repr(C)]
        pub struct pthread_attr_t {
            reserved: [u8; 16],
        }
    }
}

unsafe extern "C" {
    fn pthread_create(
        thread: *mut pthread_t,
        attr: *const pthread_attr_t,
        start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
        arg: *mut c_void
    ) -> c_int;
    fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int;
    fn pthread_detach(thread: pthread_t) -> c_int;
}

extern "C" fn start<F, T>(arg: *mut c_void) -> *mut c_void
where
    F: FnOnce() -> T
{
    let args = unsafe { Box::from_raw(arg as *mut ThreadInit<F, T>) };
    let ret = (args.thread_main)();
    unsafe { *args.ret.get() = Some(ret); }
    ptr::null_mut()
}

pub fn spawn<F, T>(args: *mut ThreadInit<F, T>) -> JoinHandle
where
    F: FnOnce() -> T
{
    let mut thread: pthread_t = 0;
    let ret = unsafe { pthread_create(
        &mut thread, // thread
        ptr::null(), // attr
        start::<F, T>, // start routine
        args as *mut c_void, // arg
    ) };
    assert!(ret == 0, "pthread_create failed: {}", Error::from_raw_os_error(ret));
    JoinHandle(Some(thread))
}

pub struct JoinHandle(Option<pthread_t>);

impl JoinHandle {
    pub fn join(mut self) {
        let thread = self.0.take().expect("attempt to join an already joined thread");
        let ret = unsafe { pthread_join(thread, ptr::null_mut()) };
        assert!(ret == 0, "pthread_join failed: {}", Error::from_raw_os_error(ret));
    }
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        if let Some(thread) = self.0 {
            let ret = unsafe { pthread_detach(thread) };
            assert!(ret == 0, "pthread_detach failed: {}", Error::from_raw_os_error(ret));
        }
    }
}

unsafe extern "C" {
    fn sysconf(name: c_int) -> c_long;
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
const _SC_NPROCESSORS_ONLN: c_int = 84;
#[cfg(target_os = "android")]
const _SC_NPROCESSORS_ONLN: c_int = 0x0061;

pub fn ncpu() -> usize {
    let ret = unsafe { sysconf(_SC_NPROCESSORS_ONLN) };
    assert!(ret != -1, "sysconf(_SC_NPROCESSORS_ONLN) failed");
    ret as usize
}
