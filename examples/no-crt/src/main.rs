#![no_std]
#![no_main]
use dstd::prelude::*;

#[cfg(windows)]
mod rlibc;

dstd::main!(main);
fn main() {
    #[cfg(windows)]
    println!("This executable does not depend on CRT library");
    #[cfg(not(windows))]
    println!("no-crt feature flag does nothing on non-windows platforms");
}
