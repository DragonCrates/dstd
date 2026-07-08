use core::panic::{PanicInfo, PanicMessage};

use crate::eprintln;

pub fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    let (file, line, column);
    if let Some(loc) = info.location() {
        file = loc.file();
        line = loc.line();
        column = loc.column();
    } else {
        file = "???";
        line = 0;
        column = 0;
    }

    panic_msg(file, line, column, message)
}

#[cfg(unix)]
fn panic_msg(file: &str, line: u32, column: u32, message: PanicMessage<'_>) -> ! {
    eprintln!("\x1b[101;37mThread panicked at {file}:{line}:{column}:\n{message}\x1b[0m");
    crate::process::exit(101)
}

#[cfg(windows)]
#[allow(nonstandard_style)]
fn panic_msg(file: &str, line: u32, column: u32, message: PanicMessage<'_>) -> ! {
    use core::ptr;
    use crate::io::stdio::{GetStdHandle, STDERR};
    use crate::prelude::{Vec, format};
    use crate::ffi::*;

    unsafe extern "C" {
        /// Displays a modal dialog box that contains a system icon, a set of buttons, and a brief application-specific message, such as status or error information.
        fn MessageBoxW(
            /* [in, optional] */ hWnd: HWND,
            /* [in, optional] */ lpText: LPCWSTR,
            /* [in, optional] */ lpCaption: LPCWSTR,
            /* [in] */ uType: UINT,
        ) -> c_int;
        /// Sets the attributes of characters written to the console screen buffer by the WriteFile or WriteConsole function, or echoed by the ReadFile or ReadConsole function. This function affects text written after the function call.
        fn SetConsoleTextAttribute(
            /* _In_ */ hConsoleOutput: HANDLE,
            /* _In_ */ wAttributes: WORD,
        ) -> BOOL;
    }
    const MB_ICONERROR: UINT = 0x00000010;
    #[repr(C)]
    #[derive(Default)]
    struct COORD {
        X: SHORT,
        Y: SHORT,
    }
    #[repr(C)]
    #[derive(Default)]
    struct SMALL_RECT {
        Left: SHORT,
        Top: SHORT,
        Right: SHORT,
        Bottom: SHORT,
    }
    #[repr(C)]
    #[derive(Default)]
    struct CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: COORD,
        dwCursorPosition: COORD,
        wAttributes: WORD,
        srWindow: SMALL_RECT,
        dwMaximumWindowSize: COORD,
    }
    type PCONSOLE_SCREEN_BUFFER_INFO = *mut CONSOLE_SCREEN_BUFFER_INFO;
    unsafe extern "C" {
        /// Retrieves information about the specified console screen buffer.
        fn GetConsoleScreenBufferInfo(
            /* _In_ */ hConsoleOutput: HANDLE,
            /* _Out_ */ lpConsoleScreenBufferInfo: PCONSOLE_SCREEN_BUFFER_INFO,
        ) -> BOOL;
    }

    let stderr = unsafe { GetStdHandle(STDERR) };
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

#[cfg(windows)]
macro_rules! w {
    ($($ch:literal),*) => {
        &[$($ch as u16),*]
    }
}
#[cfg(windows)]
use w;
