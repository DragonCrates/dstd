use core::panic::PanicMessage;

use crate::eprintln;

pub fn handle_panic(file: &str, line: u32, column: u32, message: PanicMessage<'_>) -> ! {
    eprintln!("\x1b[101;37mThread panicked at {file}:{line}:{column}:\n{message}\x1b[0m");
    crate::process::exit(101)
}
