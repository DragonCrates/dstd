//! # dstd: Dragon's standard library
//! Lightweight and feature-complete std replacement
//!
//! API should be somewhat compatible to that of std, but don't expect it to be a drop-in replacement
//!
//! For usage instructions, check documentation for [`dstd::main`](main)

#![no_std]

mod cfg_if;
pub(crate) use cfg_if::cfg_if;

pub(crate) mod panic;
pub(crate) mod sys;

// TODO:
// udp, unix sockets, tcp connect
// command spawn, pipes

pub mod alloc;
pub mod collections;
pub mod env;
pub mod fs;
pub mod init;
pub mod io;
pub mod net;
pub mod os_str;
pub mod path;
pub mod prelude;
pub mod process;
pub mod rand;
pub mod sync;
pub mod thread;
pub mod time;

#[cfg(test)]
mod tests;

// Link to libc on Linux
#[cfg(any(target_os = "linux", target_os = "android"))]
#[link(name = "c")]
unsafe extern "C" {}

// Link to winsock on Windows
#[cfg(windows)]
#[link(name = "ws2_32")]
unsafe extern "C" {}

// Needed by dstd::sync
#[cfg(windows)]
#[link(name = "synchronization")]
unsafe extern "C" {}

/// Defines a block that can be configured-out entirely
macro_rules! block {
    ($($args:tt)*) => { $($args)* };
}
pub(crate) use block;
