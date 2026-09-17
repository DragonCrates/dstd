#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::env;

dstd::main!(main);
fn main() {
    let args: Vec<_> = env::args().collect();
    println!("This program was invoked with arguments: {args:?}\n");
    println!("First 5 environment variables:");
    for (k, v) in env::vars().take(5) {
        println!("{k}={v}");
    }
    println!(); // newline
    if let Some(term) = env::var("TERM") {
        println!("Your terminal is: {term}");
    } else {
        println!("TERM variable is unset");
    }
}
