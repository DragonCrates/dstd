mod libc {
    pub use crate::sys::libc::unistd::exit;
}

pub fn exit(code: i32) -> ! {
    unsafe { libc::exit(code) }
}
