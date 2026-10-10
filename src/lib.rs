//! # dstd: Dragon's standard library
//! Lightweight and feature-complete std replacement
//!
//! API should be somewhat compatible to that of std, but don't expect it to be a drop-in replacement
//!
//! For usage instructions, check documentation for [`dstd::main`](main)

#![no_std]

mod sys;

// TODO:
// unix sockets, command spawn, pipes

pub mod alloc;
pub mod collections;
pub mod env;
pub mod fs;
pub mod init;
pub mod io;
pub mod net;
pub mod os_str;
pub mod panic;
pub mod path;
pub mod prelude;
pub mod process;
pub mod rand;
pub mod sync;
pub mod thread;
pub mod time;

mod linkage {
    #[cfg(windows)]
    crate::block! {
        // winsock
        #[link(name = "ws2_32")]
        unsafe extern "C" {}
        // synchapi (WaitOnAddress, WakeByAddress, WakeByAddressAll)
        #[link(name = "synchronization")]
        unsafe extern "C" {}
        // shell32 (CommandLineToArgvW)
        #[link(name = "shell32")]
        unsafe extern "C" {}
        // advapi32 (RtlGenRandom)
        #[link(name = "advapi32")]
        unsafe extern "C" {}
    }

    #[cfg(unix)]
    crate::block! {
        // libc
        #[link(name = "c")]
        unsafe extern "C" {}
    }
}

/// Defines a block that can be configured-out entirely
macro_rules! block {
    ($($args:tt)*) => { $($args)* };
}
use block;
