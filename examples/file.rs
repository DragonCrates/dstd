#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::io::{self, Read};
use dstd::fs::File;

dstd::main!(main);
fn main() -> io::Result<()> {
    let mut file = File::open("examples/file.rs")?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;

    println!("examples/file.rs contents:");
    println!("====================");
    print!("{}", String::from_utf8_lossy(&contents));
    println!("====================");
    Ok(())
}
