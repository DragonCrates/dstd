use core::panic::PanicInfo;

use crate::eprintln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO:
    // - MessageBoxW on windows
    // - windows uses message without ansi escapes, sets color via winapi
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
    eprintln!("\x1b[101;37mThread panicked at {file}:{line}:{column}:\n{message}\x1b[0m");
    crate::process::exit(101)
}

// Never called with panic = "abort"
#[unsafe(no_mangle)]
unsafe extern "C" fn rust_eh_personality() {}

// TODO remove if builds on windows without this. gnu and msvc
#[unsafe(no_mangle)]
unsafe extern "C" fn _Unwind_Resume() {}
