#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::{thread_local, thread};

use core::cell::Cell;

thread_local! {
    static COUNTER: Cell<u32> = Cell::new(1);
}

fn thread_fn() {
    println!("Counting to 5...");
    while COUNTER.get() <= 5 {
        println!("{}...", COUNTER.get());
        COUNTER.update(|i| i + 1);
    }
}

dstd::main!(main);
fn main() {
    let mut threads = vec![];

    println!("Launching 3 threads");
    for _ in 0..3 {
        threads.push(thread::spawn(thread_fn));
    }

    for thread in threads.into_iter() {
        println!("Join thread");
        thread.join();
    }
}
