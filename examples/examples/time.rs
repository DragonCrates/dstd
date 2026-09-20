#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::time::{SystemTime, Instant};
use dstd::thread;

dstd::main!(main);
fn main() {
    let t = SystemTime::now().to_local().unwrap();
    let weekday = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][t.weekday as usize];
    let month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][t.mon as usize];
    println!("Date: {}, {:02} {} {:04}", weekday, t.day, month, t.year);
    println!("Time: {:02}:{:02}:{:02}", t.hour, t.min, t.sec);
    println!("Timezone: UTC+{}", t.tz_offset as f32 / 3600.0);
    println!("Unix: {}", t.to_system().as_unix());

    println!("Sleep for 1 second...");
    let start = Instant::now();
    thread::usleep(1000);
    println!("Slept for: {}ms", start.elapsed().as_millis());
}
