#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec;

use dstd::sync::{Arc, Mutex};
use dstd::thread;
use dstd::println;

dstd::main!(main);
fn main() {
    let ncpu = thread::available_parallelism();
    println!("Number of threads: {ncpu}");

    let max = 1_000_000;
    let times = max / ncpu;
    let counter = Arc::new(Mutex::new(0));
    let mut threads = vec![];
    for _ in 0..ncpu {
        let counter2 = Arc::clone(&counter);
        let t = thread::spawn(move || {
            for _ in 0..times {
                *counter2.lock() += 1;
            }
        });
        threads.push(t);
    }

    for t in threads {
        t.join();
    }

    let counter_guard = counter.lock();
    assert_eq!(times * ncpu, *counter_guard);
    println!("Counter value is: {}", *counter_guard);
}
