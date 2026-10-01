use core::ptr;
use core::panic::PanicMessage;

extern crate alloc;
use alloc::vec::Vec;
use alloc::format;

use crate::eprintln;
use crate::sys::windows::consoleapi2::*;
use crate::sys::windows::winuser::*;
use crate::sys::windows::processenv::GetStdHandle;
use crate::sys::windows::winbase::STD_ERROR_HANDLE;

pub fn handle_panic(file: &str, line: u32, column: u32, message: PanicMessage<'_>) -> ! {
    let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    if stderr.is_null() {
        // No stderr, call MessageBoxW
        let msg: Vec<_> = format!("Thread panicked at {file}:{line}:{column}:\r\n{message}\0").encode_utf16().collect();

        const RUST_PANIC: &[u16] = w!('R', 'u', 's', 't', ' ', 'p', 'a', 'n', 'i', 'c', '\0');
        unsafe { MessageBoxW(
            ptr::null_mut(), // hWnd
            msg.as_ptr(), // lpText
            RUST_PANIC.as_ptr(), // lpCaption
            MB_ICONERROR, // uType
        ); }
    } else {
        // Console handle is not null
        let mut old = CONSOLE_SCREEN_BUFFER_INFO::default();
        unsafe { GetConsoleScreenBufferInfo(stderr, &mut old); }
        unsafe { SetConsoleTextAttribute(stderr, 0xcf); }
        eprintln!("Thread panicked at {file}:{line}:{column}:\r\n{message}");
        unsafe { SetConsoleTextAttribute(stderr, old.wAttributes); }
    }

    crate::process::exit(101)
}

// If we need more wide strings, we could do similar: https://github.com/rust-lang/rust/blob/1625626af12c1f95540530c6fc1242ff10aaba0a/library/std/src/sys/pal/windows/api.rs#L38
macro_rules! w {
    ($($ch:literal),*) => {
        &[$($ch as u16),*]
    }
}
use w;
