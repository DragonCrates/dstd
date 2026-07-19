#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub mod libc;
