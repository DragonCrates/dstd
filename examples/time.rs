#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::time::{SystemTime, FormatTime, Instant};
use dstd::thread;

dstd::main!(main);
fn main() {
    let t = SystemTime::now().to_local().unwrap();
    let weekday = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][t.weekday as usize];
    let month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][t.mon as usize];
    println!("Date: {}, {:02} {} {:04}", weekday, t.day, month, t.year);
    println!("Time: {:02}:{:02}:{:02}", t.hour, t.min, t.sec);
    println!("Timezone: UTC+{}", t.tz_offset as f32 / 3600.0);
    println!("Unix: {}", t.to_system().to_unix());

    println!("Sleep for 1 second...");
    let start = Instant::now();
    thread::usleep(1000);
    println!("Slept for: {}ms", start.elapsed().as_millis());

    println!("Running tests...");
    // 1740607859 = Wed 26 Feb 2025 22:10:59 GMT
    let t = SystemTime::from_unix(1740607859).to_global().unwrap();
    assert_eq!(t, FormatTime {
        year: 2025,
        mon: 1, // Feb
        day: 26,
        hour: 22,
        min: 10,
        sec: 59,
        weekday: 3, // Wed
        tz_offset: 0,
    });

    // 1767481606 = Sat 03 Jan 2026 23:06:46 GMT
    let t = FormatTime {
        year: 2026,
        mon: 0,
        day: 3,
        hour: 23,
        min: 6,
        sec: 46,
        weekday: 0,
        tz_offset: 0,
    };
    assert_eq!(1767481606, t.to_system().to_unix());

    let t = FormatTime {
        year: 1969,
        mon: 11,
        day: 31,
        hour: 23,
        min: 59,
        sec: 59,
        weekday: 3,
        tz_offset: 0,
    };
    assert_eq!(-1, t.to_system().to_unix());
    println!("Done!");
}
