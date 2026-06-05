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
    use crate::io::stdio::{MultiByteToWideChar, GetStdHandle, STDERR};
    use crate::prelude::{vec, format};
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
        let msg = format!("Thread panicked at {file}:{line}:{column}:\r\n{message}\0");
        let mut wstr = vec![0_u16; msg.len()];
        let ret = unsafe { MultiByteToWideChar(
            65001, // CodePage (CP_UTF8)
            0, // dwFlags
            msg.as_ptr() as LPCCH, // lpMultiByteStr
            msg.len() as c_int, // cbMultiByte
            wstr.as_mut_ptr(), // lpWideCharStr
            wstr.len() as c_int, // cchWideChR
        ) };

        if ret == 0 {
            // This is an error message for an error message... You win!
            const MBTOWC_FAILED: &[u16] = &['M' as u16, 'u' as u16, 'l' as u16, 't' as u16, 'i' as u16, 'B' as u16, 'y' as u16, 't' as u16, 'e' as u16, 'T' as u16, 'o' as u16, 'W' as u16, 'i' as u16, 'd' as u16, 'e' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'r' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, '\0' as u16];
            wstr = MBTOWC_FAILED.to_vec();
        }

        const RUST_PANIC: &[u16] = &['R' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16, 'p' as u16, 'a' as u16, 'n' as u16, 'i' as u16, 'c' as u16, '\0' as u16];
        unsafe { MessageBoxW(
            ptr::null_mut(), // hWnd
            wstr.as_ptr(), // lpText
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
