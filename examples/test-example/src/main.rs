#![no_std]
// To make tests work, you have to make the no_main attribute optional
#![cfg_attr(not(test), no_main)]
use dstd::prelude::*;

dstd::main!(main);
fn main() {
    println!("This example supports `cargo test` with dstd");
}

#[test]
#[should_panic]
fn failing_test_case() {
    println!("All dstd features work");
    panic!("This is an example of a failing test case");
}
