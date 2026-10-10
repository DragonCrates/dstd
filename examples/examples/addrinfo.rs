#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::io;
use dstd::net::ToSocketAddrs;

dstd::main!(main);
fn main() -> io::Result<()> {
    let hostname = "ya.ru:443";

    println!("Resolving {hostname}");
    for addr in hostname.to_socket_addrs()? {
        println!("- {}", addr);
    }

    Ok(())
}
