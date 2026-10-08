use core::sync::atomic::{AtomicI64, Ordering};
use core::ffi::c_long;
use core::ops::{Add, Sub};

use crate::sys::windows::minwindef::*;
use crate::sys::windows::minwinbase::*;
use crate::sys::windows::corecrt::errno_t;
use super::{time_t, Tm, TimeDelta};

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
    /// Retrieves the current system date and time. The information is in Coordinated Universal Time (UTC) format.
    fn GetSystemTimeAsFileTime(
        /* [out] */ lpSystemTimeAsFileTime: LPFILETIME
    );
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(LARGE_INTEGER);

const NANOS_PER_SEC: i64 = 1_000_000_000;
impl Instant {
    pub fn now() -> Instant {
        let mut li: LARGE_INTEGER = 0;
        unsafe { QueryPerformanceCounter(&mut li); }
        Instant(li)
    }

    pub fn duration_since(&self, earlier: Instant) -> TimeDelta {
        Instant(self.0 - earlier.0).as_duration()
    }

    fn as_duration(&self) -> TimeDelta {
        let freq = get_freq();
        let secs = self.0 / freq;
        let nanos = (self.0 % freq) * NANOS_PER_SEC / freq;
        TimeDelta::new(secs, nanos as u32)
    }

    fn from_duration(dur: TimeDelta) -> Instant {
        let secs = dur.as_secs();
        let nanos = dur.subsec_nanos() as i64;
        let freq = get_freq();
        Instant(secs * freq + nanos * freq / NANOS_PER_SEC)
    }
}

impl Add<TimeDelta> for Instant {
    type Output = Instant;
    fn add(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 + Instant::from_duration(rhs).0)
    }
}

impl Sub<TimeDelta> for Instant {
    type Output = Instant;
    fn sub(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 - Instant::from_duration(rhs).0)
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

pub fn sleep(dur: TimeDelta) {
    unsafe { Sleep(dur.as_millis() as DWORD); }
}

const WINDOWS_TICK: i64 = 10_000_000;
const SEC_TO_UNIX_EPOCH: i64 = 11644473600;
fn filetime_to_timedelta(ft: FILETIME) -> TimeDelta {
    let t = (ft.dwHighDateTime as i64) << 32 | ft.dwLowDateTime as i64;
    let relative_to_unix_100ns = t - SEC_TO_UNIX_EPOCH * WINDOWS_TICK;
    let secs = relative_to_unix_100ns / WINDOWS_TICK;
    let nanos = relative_to_unix_100ns % WINDOWS_TICK * 100;
    TimeDelta::new_normalized(secs, nanos as u32)
}

pub fn time() -> TimeDelta {
    let mut out = FILETIME::default();
    unsafe { GetSystemTimeAsFileTime(&mut out); }
    filetime_to_timedelta(out)
}
