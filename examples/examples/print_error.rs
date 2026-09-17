#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::io;
use dstd::net::TcpListener;

dstd::main!(main);
fn main() -> io::Result<()> {
    println!("This will print error if bind on port 80 fails");
    TcpListener::bind("0.0.0.0:80".parse().unwrap())?;
    println!("Bind successful!");
    Ok(())
}
