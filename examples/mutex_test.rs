#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::sync::{Arc, Mutex};
use dstd::thread;

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
    if *counter_guard != max {
        println!("Test FAILED!");
    } else {
        println!("Test PASSED!");
    }
    // TODO does MutexGuard implement Debug/Display?
    println!("Counter value is: {}", *counter_guard);
}
