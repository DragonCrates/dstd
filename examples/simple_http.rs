#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::io::{self, Write};
use dstd::net::{TcpListener, TcpStream};
use dstd::thread;

dstd::main!(main);
fn main() -> io::Result<()> {
    let server = TcpListener::bind("0.0.0.0:8080".parse().unwrap())?;
    loop {
        let (client, addr) = server.accept()?;
        println!("Request from {addr}");
        thread::spawn(move || {
            let _ = handle_client(client);
        });
    }
}

fn handle_client(mut client: TcpStream) -> io::Result<()> {
    client.write_all(concat!(
        "HTTP/1.1 200 OK\r\n",
        "Connection: close\r\n",
        "Content-Length: 14\r\n",
        "\r\n",
        "dstd says hi!\n",
    ).as_bytes())?;
    Ok(())
}
