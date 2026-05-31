use core::time::Duration;
use core::sync::atomic::{AtomicI64, Ordering};

use crate::ffi::*;
use super::{time_t, Tm};

unsafe extern "C" {
    /// Retrieves the current value of the performance counter
    fn QueryPerformanceCounter(
        /* [out] */ lpPerformanceCount: *mut LARGE_INTEGER
    ) -> BOOL;
    /// Retrieves the frequency of the performance counter
    fn QueryPerformanceFrequency(
        /* [out] */ lpFrequency: *mut LARGE_INTEGER
    ) -> BOOL;
    /// Converts a time_t time value to a tm structure
    fn _gmtime64_s(tmDest: *mut Tm, sourceTime: *const time_t) -> errno_t;
    /// Converts a time value and corrects for the local time zone
    fn _localtime64_s(tmDest: *mut Tm, sourceTime: *const time_t) -> errno_t;
    /// Retrieves the difference in seconds between coordinated universal time (UTC) and local time
    fn _get_timezone(seconds: *mut c_long) -> errno_t;
}

static FREQ: AtomicI64 = AtomicI64::new(0);
/// Returns number of ticks per second for QPC
fn get_freq() -> LARGE_INTEGER {
    let mut freq = FREQ.load(Ordering::Relaxed);

    if freq == 0 {
        // Initialize
        unsafe { QueryPerformanceFrequency(&mut freq); }
        FREQ.store(freq, Ordering::Relaxed);
    }

    freq
}

#[derive(Debug, Clone, Copy)]
pub struct Instant(LARGE_INTEGER);

impl Instant {
    pub fn now() -> Instant {
        let mut li: LARGE_INTEGER = 0;
        unsafe { QueryPerformanceCounter(&mut li); }
        Instant(li)
    }

    pub fn duration_since(self, earlier: Instant) -> Duration {
        let diff = self.0 - earlier.0;
        let freq_us = get_freq() / 1_000_000;
        Duration::from_micros((diff / freq_us) as u64)
    }
}

pub fn gmtime(time: time_t) -> Option<Tm> {
    unsafe {
        let mut tm = Tm::default();
        let ret = _gmtime64_s(&mut tm, &time);
        if ret != 0 { return None; }
        Some(tm)
    }
}

pub fn localtime(time: time_t) -> Option<Tm> {
    unsafe {
        let mut tm = Tm::default();
        let ret = _localtime64_s(&mut tm, &time);
        if ret != 0 { return None; }

        let mut gmtoff = 0;
        let ret = _get_timezone(&mut gmtoff);
        if ret != 0 { return None; }
        // Windows does not have such field in struct tm
        tm.tm_gmtoff = gmtoff;
        Some(tm)
    }
}

unsafe extern "C" {
    fn Sleep(
        /* [in] */ dwMilliseconds: DWORD
    );
}

pub fn sleep(dur: Duration) {
    unsafe { Sleep(dur.as_millis() as DWORD); }
}
