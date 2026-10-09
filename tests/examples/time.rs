#![no_std]
#![no_main]

extern crate alloc;

use dstd::time::{Instant, SystemTime, TimeDelta};
use dstd::thread;
use dstd::println;

dstd::main!(main);
fn main() {
    // 1. Instant: monotonic, and elapsed after a sleep
    let a = Instant::now();
    let b = Instant::now();
    assert!(b >= a, "Instant::now() must be monotonic");

    thread::usleep(100);
    let c = Instant::now();
    let ms = (c - a).as_millis();
    println!("elapsed after 100ms sleep: {ms}ms");
    assert!(ms >= 100 && ms < 2000, "elapsed after 100ms sleep is {ms}ms");

    // 2. Instant +/- TimeDelta
    let half = TimeDelta::from_millis(500);
    let plus = a + half;
    let back = plus - half;
    let ms = (plus - a).as_millis();
    assert!((490..=510).contains(&ms), "a+500ms measured {ms}ms");
    println!("a+500ms = {ms}ms, a+500ms-500ms ~= a");
    let drift = (back.duration_since(a)).as_secs().abs();
    assert!(drift <= 1, "add/sub roundtrip drifted {drift}s");
    println!("add/sub roundtrip drift = {drift}s");

    // 3. SystemTime: epoch sanity + FILETIME sub-second precision
    let st = SystemTime::now();
    let ts = st.as_unix();
    println!("SystemTime unix ts = {ts}, subsec_nanos = {}", st.as_duration().subsec_nanos());
    assert!(ts > 1_750_000_000 && ts < 2_000_000_000, "unix timestamp {ts} out of expected range");
    let from_dur = SystemTime::from_duration(st.as_duration());
    assert_eq!(from_dur.as_unix(), ts, "from_duration/as_duration round trip");

    // 4. Global vs local formatting
    let g = st.to_global().unwrap();
    let l = st.to_local().unwrap();
    println!("global: {g:?}");
    println!("local : {l:?}");
    assert_eq!(g.tz_offset, 0, "to_global must report UTC offset 0");

    // Wall clock difference (seconds of day) must equal the offset, modulo 24h.
    // Comparing days/months directly is invalid across the date boundary
    // (e.g. UTC is still Oct 8 while a UTC+10 zone is already Oct 9).
    let g_sod = (g.hour as i64) * 3600 + (g.min as i64) * 60 + g.sec as i64;
    let l_sod = (l.hour as i64) * 3600 + (l.min as i64) * 60 + l.sec as i64;
    let sod_diff = (l_sod - g_sod - l.tz_offset as i64).rem_euclid(86_400);
    println!("local offset = {}s, wall-clock diff (mod 24h) = {sod_diff}s", l.tz_offset);
    assert_eq!(sod_diff, 0, "local wall clock disagrees with offset: {l:?} vs {g:?}");

    // 5. global -> system and local -> system round trips
    let g_rt = g.to_system();
    let l_rt = l.to_system();
    let g_diff = (st.as_unix() - g_rt.as_unix()).abs();
    let l_diff = (st.as_unix() - l_rt.as_unix()).abs();
    println!("global roundtrip diff = {g_diff}s, local roundtrip diff = {l_diff}s");
    assert!(g_diff <= 2, "to_global->to_system round trip off by {g_diff}s");
    assert!(l_diff <= 2, "to_local->to_system round trip off by {l_diff}s");

    // 5b. Day-boundary case: UTC is 23:30 on Oct 8, but in an east-of-UTC zone
    //     the local date has already rolled over to Oct 9. This must still
    //     round-trip exactly.
    let boundary = SystemTime::from_unix(1791502200);
    let gb = boundary.to_global().unwrap();
    let lb = boundary.to_local().unwrap();
    assert_eq!(gb.day, 8, "UTC date should be Oct 8");
    let gb_sod = (gb.hour as i64) * 3600 + (gb.min as i64) * 60 + gb.sec as i64;
    let lb_sod = (lb.hour as i64) * 3600 + (lb.min as i64) * 60 + lb.sec as i64;
    let sod_diff_b = (lb_sod - gb_sod - lb.tz_offset as i64).rem_euclid(86_400);
    assert_eq!(sod_diff_b, 0, "boundary wall clock disagrees: {lb:?} vs {gb:?}");
    let l_rt_b = lb.to_system();
    let l_diff_b = (boundary.as_unix() - l_rt_b.as_unix()).abs();
    assert!(l_diff_b <= 2, "boundary local roundtrip off by {l_diff_b}s");
    println!("day-boundary ok: global day {}, local day {}", gb.day, lb.day);

    // 6. negative differences must stay negative (normalized secs/nanos)
    let future = Instant::now() + TimeDelta::from_millis(100);
    let negative = Instant::now() - future;
    let negative_ms = negative.as_millis();
    println!("now - future(100ms) = {negative_ms}ms");
    assert!(negative_ms < 0, "duration towards a future instant must be negative, got {negative_ms}ms");
    assert!(negative_ms > -2000, "negative duration implausibly large: {negative_ms}ms");

    println!("All tests passed");
}
