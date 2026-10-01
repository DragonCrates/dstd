use core::ffi::c_int;

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    // Linux fcntl
    pub const O_RDONLY: c_int = 0;
    pub const O_WRONLY: c_int = 1;
    pub const O_RDWR: c_int = 2;
    pub const O_CREAT: c_int = 0o100;
    pub const O_EXCL: c_int = 0o200;
    pub const O_TRUNC: c_int = 0o1000;
    pub const O_APPEND: c_int = 0o2000;
    pub const O_NONBLOCK: c_int = 0o4000;
    pub const O_CLOEXEC: c_int = 0o2000000;
}

unsafe extern "C" {
    /// Open and possibly create a file
    pub fn open(path: *const u8, flags: c_int, ...) -> c_int;
}
