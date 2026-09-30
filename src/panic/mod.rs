//! Panic utilities

use core::panic::PanicInfo;

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

/// The default panic handler
///
/// Used by [`dstd::main`](crate::main)
pub fn handle_panic(info: &PanicInfo) -> ! {
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

    sys::handle_panic(file, line, column, message)
}
