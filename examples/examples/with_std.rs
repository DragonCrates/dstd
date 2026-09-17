// You can still use std without any issues if you are not adding
// `dstd::main!` and `#![no_std]`
fn main() {
    std::println!("This is print from std");
    dstd::println!("This is print from dstd");
}
