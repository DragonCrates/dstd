#![no_std]
#![no_main]

extern crate alloc;
use alloc::collections::VecDeque;

use dstd::sync::{Arc, Mutex, Condvar};
use dstd::thread;
use dstd::println;

struct Channel {
    queue: Mutex<VecDeque<u32>>,
    cond: Condvar,
}

dstd::main!(main);
fn main() {
    let channel = Arc::new(Channel {
        queue: Mutex::new(VecDeque::new()),
        cond: Condvar::new(),
    });

    let last = 25;

    let channel2 = Arc::clone(&channel);
    let t = thread::spawn(move || {
        println!("Locking channel");
        let mut guard = channel2.queue.lock();
        println!("Locked");
        'out: loop {
            while let Some(val) = guard.pop_front() {
                println!("Consumed {val}");
                if val == last {
                    println!("Received last item! Done");
                    break 'out
                }
            }
            println!("No more items, wait for notification");
            guard = channel2.cond.wait(guard);
            println!("Received notification");
        }
    });

    for i in 1..=last {
        println!("Producing {i}");
        channel.queue.lock().push_back(i);
        println!("Send notification");
        channel.cond.notify_one();
        thread::usleep(50);
    }

    t.join();
}
