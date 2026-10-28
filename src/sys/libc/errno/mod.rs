#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    mod linux;
    pub use linux::*;
}
