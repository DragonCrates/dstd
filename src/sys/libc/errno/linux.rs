use core::ffi::c_int;

// Linux errno
pub const EINTR: i32 = 4;
pub const EWOULDBLOCK: i32 = 11;
pub const EAGAIN: i32 = EWOULDBLOCK;
pub const EISDIR: i32 = 21;

unsafe extern "C" {
    /// Returns the address of the calling thread's `errno` storage.
    #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
    #[cfg_attr(target_os = "android", link_name = "__errno")]
    pub fn errno() -> *mut c_int;
}
