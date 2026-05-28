//! # dstd: Dragon's standard library
//! Lightweight and feature-complete std replacement
//!
//! API should be somewhat compatible to that of std, but don't expect it to be a drop-in replacement
//! # Usage
//! Set up your Cargo.toml:
//! ```
//! [profile.dev]
//! panic = "abort"
//!
//! [profile.release]
//! panic = "abort"
//! ```
//! Then, follow this example:
//! ```
//! #![no_std]
//! #![no_main]
//! use dstd::prelude::*;
//!
//! dstd::main!(main);
//! fn main() {
//!     println!("Hello world");
//! }
//! ```
//!
//! After that, you are all set

#![no_std]

#![allow(clippy::needless_return)]

mod cfg_if;
pub(crate) use cfg_if::{block, cfg_if};

mod alloc;
mod panic;

// TODO:
// fs
// stdin
// udp, unix sockets, tcp connect
// command spawn, pipes

pub mod io;
// TODO fs
pub mod net;
pub mod thread;
pub mod time;
pub mod sync;
pub mod process;
pub mod prelude;
pub mod init;
pub mod env;
pub mod ffi;

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
