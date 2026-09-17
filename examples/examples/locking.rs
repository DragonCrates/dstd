#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::sync::{Arc, Mutex};
use dstd::thread;

dstd::main!(main);
fn main() {
    let mtx = Arc::new(Mutex::new(()));
    let _lock0 = mtx.lock();
    println!("Lock acquired!");
    drop(_lock0);
    println!("Lock released!");
    let _lock1 = mtx.lock();
    println!("Lock acquired!");

    let mtx2 = Arc::clone(&mtx);
    thread::spawn(move || {
        println!("I want that lock too!");
        let _lock3 = mtx2.lock();
        println!("Thanks!");
    });

    thread::usleep(100);
    println!("Okay, take it!");
    drop(_lock1);

    let _lock2 = mtx.lock();
    println!("Lock acquired!");
}
