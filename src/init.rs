use core::error::Error;
use core::slice;

use crate::println;
use crate::env::ARGS;

// TODO: windows.rs, unix.rs
#[cfg(windows)]
crate::block! {
    use core::ffi::c_int;
    use crate::sys::windows::winnt::*;
    use crate::sys::windows::consoleapi::*;
    use crate::sys::windows::winnls::CP_UTF8;
    use crate::sys::windows::sysinfoapi::GetVersion;

    unsafe extern "system" {
        fn GetCommandLineW() -> LPWSTR;
        fn CommandLineToArgvW(
            /* [in] */ lpCmdLine: LPCWSTR,
            /* [out] */ pNumArgs: *mut c_int,
        ) -> *const LPWSTR;
    }

    #[allow(non_snake_case)]
    fn is_win7() -> bool {
        let dwVersion = unsafe { GetVersion() };
        let dwMajorVersion = dwVersion & 0x00FF;
        let dwMinorVersion = (dwVersion & 0xFF00) >> 8;
        dwMajorVersion <= 6 && dwMinorVersion <= 1
    }

    /// Not a public api! Exposed only for the `main!` macro
    #[doc(hidden)]
    pub unsafe fn __init(_argc: i32, _argv: *const *const u8) {
        unsafe {
            let mut argc = 0;
            let argv = CommandLineToArgvW(GetCommandLineW(), &mut argc);
            ARGS = slice::from_raw_parts(argv as *const *const _, argc as usize);
            if !is_win7() {
                SetConsoleCP(CP_UTF8);
                SetConsoleOutputCP(CP_UTF8);
                crate::println!("Enabled UTF-8 console");
            }
        }
    }
}

#[cfg(unix)]
crate::block! {
    use core::ffi::c_int;

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

/// Defines a main function for your crate
/// # Usage
/// First, set up your Cargo.toml:
/// ```toml
/// [profile.dev]
/// panic = "abort"
///
/// [profile.release]
/// # You have to disable panic unwinding because unwinding is not supported without std
/// panic = "abort"
/// # Other recommended options
/// lto = "thin"
/// strip = true
/// ```
/// Then, follow this example:
/// ```ignore
#[doc = include_str!("../examples/examples/helloworld.rs")]
/// ```
/// After that, you are all set
#[macro_export]
macro_rules! main {
    ($name:ident) => {
        #[cfg(not(test))]
        mod __dstd_main {
            use core::panic::PanicInfo;
            use $crate::alloc::System;
            use $crate::init::Termination;

            #[global_allocator]
            static GLOBAL_ALLOC: System = System;

            #[panic_handler]
            fn panic_handler(info: &PanicInfo) -> ! {
                $crate::panic::handle_panic(info)
            }

            $crate::init::__make_main!($name);

            // Never called with panic = "abort"
            #[unsafe(no_mangle)]
            unsafe extern "C" fn rust_eh_personality() {}

            // glibc, musl and mingw targets need this
            #[cfg(any(target_env = "gnu", target_env = "musl"))]
            #[unsafe(no_mangle)]
            unsafe extern "C" fn _Unwind_Resume() {}
        }
    }
}
pub use main;

/// Not a public api! Exposed only for the `main!` macro
#[cfg(not(all(windows, feature = "no-crt")))]
#[doc(hidden)]
#[macro_export]
macro_rules! __make_main {
    ($name:ident) => {
        #[unsafe(no_mangle)]
        unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
            unsafe { $crate::init::__init(argc, argv); }
            super::$name().report()
        }
    }
}

/// Not a public api! Exposed only for the `main!` macro
#[cfg(all(windows, feature = "no-crt"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __make_main {
    ($name:ident) => {
        #[unsafe(no_mangle)]
        unsafe extern "C" fn mainCRTStartup() {
            unsafe { $crate::init::__init(0, core::ptr::null()); }
            let exit_code = super::$name().report();
            $crate::process::exit(exit_code)
        }
    }
}

#[doc(hidden)]
pub use __make_main;

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

impl Sealed for i32 {}
impl Termination for i32 {
    fn report(&self) -> i32 {
        *self
    }
}
