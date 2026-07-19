//! Definitions for items that should be accessible across entire crate

#![allow(non_camel_case_types)]

pub type c_ssize_t = isize;
pub type c_size_t = usize;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod errno {
    use core::ffi::c_int;

    pub const EINTR: i32 = 4;
    pub const EAGAIN: i32 = 11;

    unsafe extern "C" {
        /// Returns the address of the calling thread's `errno` storage.
        #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
        #[cfg_attr(target_os = "android", link_name = "__errno")]
        pub fn errno() -> *mut c_int;
    }
}
