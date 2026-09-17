extern crate alloc;
use alloc::vec;

use crate::sync::{Arc, Mutex};
use crate::thread;

/// Test that [`Mutex`] works as expected
#[test]
fn mutex_test() {
    let ncpu = thread::available_parallelism();
    //println!("Number of threads: {ncpu}");

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

    assert_eq!(max, *counter.lock());

    //println!("Counter value is: {}", *counter_guard);
}
