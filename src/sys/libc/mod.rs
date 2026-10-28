#![allow(non_camel_case_types)]

use core::ffi::c_int;

pub type Fd = c_int;
pub type c_ssize_t = isize;
pub type c_size_t = usize;

pub mod errno;
pub mod fcntl;
pub mod socket;
pub mod unistd;
