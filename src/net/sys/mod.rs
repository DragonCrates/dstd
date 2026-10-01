#[cfg(windows)]
crate::block! {
    mod windows;
    pub use windows::*;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    pub use unix::*;
}
