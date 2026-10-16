#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use dstd::io::{BufReader, Read, Result};
use dstd::println;

dstd::main!(main);

/// Underlying reader with a fixed amount of data
struct Mock {
    data: Vec<u8>,
    pos: usize,
}

impl Mock {
    fn new(data: &[u8]) -> Mock {
        Mock { data: data.to_vec(), pos: 0 }
    }
}

impl Read for Mock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let avail = &self.data[self.pos..];
        let n = avail.len().min(buf.len());
        buf[..n].copy_from_slice(&avail[..n]);
        self.pos += n;
        Ok(n)
    }
}

fn main() {
    // 1. read() must report how many bytes it copied after refilling the
    // internal buffer, not Ok(0) (false EOF) - destination smaller than data
    let mut br = BufReader::new(Mock::new(&[0xAA; 100]));
    let mut out = [0u8; 32];
    let n = br.read(&mut out).unwrap();
    assert_eq!(n, 32, "read() returned {n}, expected 32");
    assert!(out.iter().all(|&b| b == 0xAA), "read() did not fill the destination");
    println!("read() copied 32 bytes and reported 32");

    // 2. read() must not panic when the refilled internal buffer holds
    // less data than the destination has space for
    let mut br = BufReader::new(Mock::new(b"hello world"));
    let mut out = [0u8; 32];
    let n = br.read(&mut out).unwrap();
    assert_eq!(n, 11, "read() returned {n}, expected 11");
    assert_eq!(&out[..11], b"hello world");
    println!("read() copied 11 of 32 destination bytes without panicking");

    // read_to_end exercises the same refill path with a larger destination
    let mut br = BufReader::new(Mock::new(b"hello world"));
    let mut v = Vec::new();
    br.read_to_end(&mut v).unwrap();
    assert_eq!(v, b"hello world");
    println!("read_to_end got all bytes");
}
