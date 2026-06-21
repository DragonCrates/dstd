use core::error::Error;
use core::slice;

use crate::println;
use crate::env::ARGS;
use crate::ffi::*;

#[cfg(windows)]
crate::block! {
    unsafe extern "C" {
        fn GetCommandLineW() -> LPWSTR;
        fn CommandLineToArgvW(
            /* [in] */ lpCmdLine: LPCWSTR,
            /* [out] */ pNumArgs: *mut c_int,
        ) -> *const LPWSTR;
    }

    /// Not a public api! Exposed only for the `main!` macro
    #[doc(hidden)]
    pub unsafe fn __init(_argc: i32, _argv: *const *const u8) {
        unsafe {
            let mut argc = 0;
            let argv = CommandLineToArgvW(GetCommandLineW(), &mut argc);
            ARGS = slice::from_raw_parts(argv, argc as usize);
        }
    }
}

#[cfg(unix)]
crate::block! {
    unsafe extern "C" {
        fn signal(signum: c_int, handler: usize) -> usize;
    }
    const SIGPIPE: c_int = 13;
    const SIG_IGN: usize = 1;

    /// Not a public api! Exposed only for the `main!` macro
    #[doc(hidden)]
    pub unsafe fn __init(argc: i32, argv: *const *const u8) {
        unsafe {
            // sigpipe handler
            signal(SIGPIPE, SIG_IGN);
            // set args vector
            ARGS = slice::from_raw_parts(argv, argc as usize);
        }
    }
}

/// Not a public api! Exposed only for the `main!` macro
#[doc(hidden)]
pub use crate::alloc::System as __System;
/// Not a public api! Exposed only for the `main!` macro
#[doc(hidden)]
pub use crate::panic::panic as __panic;

/// Defines a main function for your crate
///
/// See the [module level](crate) doc for example
#[macro_export]
macro_rules! main {
    ($name:ident) => {
        mod __dstd_main {
            use core::panic::PanicInfo;
            use $crate::init::{Termination, __System, __panic};

            #[global_allocator]
            static GLOBAL_ALLOC: __System = __System;

            #[panic_handler]
            fn panic_handler(info: &PanicInfo) -> ! {
                __panic(info)
            }

            #[unsafe(no_mangle)]
            fn main(argc: i32, argv: *const *const u8) -> i32 {
                unsafe { $crate::init::__init(argc, argv); }
                super::$name().report()
            }

            // Never called with panic = "abort"
            #[unsafe(no_mangle)]
            unsafe extern "C" fn rust_eh_personality() {}

            // TODO remove if builds on windows without this. gnu and msvc
            //#[unsafe(no_mangle)]
            //unsafe extern "C" fn _Unwind_Resume() {}
        }
    }
}
pub use main;

mod private {
    pub trait Sealed {}
}
use private::Sealed;

pub trait Termination: Sealed {
    fn report(&self) -> i32;
}

impl Sealed for () {}
impl Termination for () {
    fn report(&self) -> i32 {
        0
    }
}

impl<T, E> Sealed for Result<T, E> {}
impl<T, E: Error> Termination for Result<T, E> {
    fn report(&self) -> i32 {
        match self {
            Ok(_) => 0,
            Err(err) => {
                println!("Error: {err:?}");
                1
            }
        }
    }
}
