//! Time functions

use core::ptr;
use core::ffi::{c_int, c_long, c_char};

#[cfg(windows)]
crate::block! {
    mod windows;
    use windows as sys;
}

#[cfg(unix)]
crate::block! {
    mod unix;
    use unix as sys;
}

unsafe extern "C" {
    fn time(tloc: *mut time_t) -> time_t;
}

#[doc(no_inline)]
pub use core::time::{Duration, TryFromFloatSecsError};

/// Monotonic clock, used to measure time
#[derive(Debug, Clone, Copy)]
pub struct Instant(sys::Instant);

impl Instant {
    /// Returns current time
    pub fn now() -> Instant {
        Instant(sys::Instant::now())
    }

    /// Returns the amount of time elapsed since this instant
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(*self)
    }

    /// Returns the amount of time elapsed from another instant to this one
    ///
    /// Equivalent: `self - earlier`
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.duration_since(earlier.0)
    }
    // TODO: checked_duration_since

    // TODO: as_duration, from_duration

    // TODO: checked_add, checked_sub
}

// TODO: impl Sub Instand and Add/Sub Duration

/// System clock, that represents real world time
#[derive(Default, Debug, Clone, Copy)]
pub struct SystemTime {
    time: time_t,
}

impl SystemTime {
    /// Returns current real time
    pub fn now() -> SystemTime {
        unsafe { SystemTime {
            time: time(ptr::null_mut())
        } }
    }

    /// Formats this time, using global timezone
    pub fn to_global(&self) -> Option<FormatTime> {
        let tm = sys::gmtime(self.time)?;
        Some(FormatTime::from_tm(tm))
    }

    /// Formats this time, using local timezone
    pub fn to_local(&self) -> Option<FormatTime> {
        let tm = sys::localtime(self.time)?;
        Some(FormatTime::from_tm(tm))
    }

    /// Returns unix timestamp (`time_t`)
    pub fn as_unix(&self) -> time_t {
        self.time
    }

    /// Constructs `SystemTime` from a unix timestamp
    pub fn from_unix(time: time_t) -> SystemTime {
        SystemTime { time }
    }

    // TODO: as_duration, from_duration
    // Probably need to expose struct timespec or create new duration
    // Or just expost .as_nanos
}

/// Formatted time structure
#[derive(Default, Debug, Clone, Copy)]
pub struct FormatTime {
    /// Year
    pub year: i32,
    /// [0, 11] Month (January = 0)
    pub mon: u8,
    /// [1, 31] Day of the month
    pub day: u8,
    /// [0, 23] Hour
    pub hour: u8,
    /// [0, 59] Minutes
    pub min: u8,
    /// [0, 60] Seconds
    pub sec: u8,
    /// [0, 6] Day of the week (Sunday = 0)
    pub weekday: u8,
    /// Timezone offset in seconds
    pub tz_offset: i32,
}

impl FormatTime {
    /// Converts `FormatTime` to a `SystemTime`
    pub fn to_system(&self) -> SystemTime {
        let hms = hms_to_time(self.hour, self.min, self.sec);
        let ymd = epoch_days_fast(self.year, self.mon+1, self.day) * 86400;
        let time = hms + ymd - self.tz_offset as time_t;
        SystemTime { time }
    }

    pub(crate) fn from_tm(tm: Tm) -> FormatTime {
        FormatTime {
            year: tm.tm_year + 1900,
            mon: tm.tm_mon as u8,
            day: tm.tm_mday as u8,
            hour: tm.tm_hour as u8,
            min: tm.tm_min as u8,
            sec: tm.tm_sec as u8,
            weekday: tm.tm_wday as u8,
            tz_offset: tm.tm_gmtoff as i32,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct Tm {
    /// [0, 60] Seconds
    pub tm_sec: c_int,
    /// [0, 59] Minutes
    pub tm_min: c_int,
    /// [0, 23] Hour
    pub tm_hour: c_int,
    /// [1, 31] Day of the month
    pub tm_mday: c_int,
    /// [0, 11] Month (January = 0)
    pub tm_mon: c_int,
    /// Year minus 1900
    pub tm_year: c_int,
    /// [0, 6] Day of the week (Sunday = 0)
    pub tm_wday: c_int,
    /// [0, 365] Day of the year (Jan/01 = 0)
    pub tm_yday: c_int,
    /// Daylight savings flag
    pub tm_isdst: c_int,
    /// Seconds East of UTC
    pub tm_gmtoff: c_long,
    /// Timezone abbreviation
    pub tm_zone: *const c_char,
}

/// 64-bit signed integer that represents a Unix timestamp
#[allow(non_camel_case_types)]
pub type time_t = i64;

pub(crate) fn sleep(dur: Duration) {
    sys::sleep(dur);
}

// Thanks for inspiration
// https://blog.reverberate.org/2020/05/12/optimizing-date-algorithms.html
fn hms_to_time(h: u8, m: u8, s: u8) -> time_t {
    (h as time_t * 3600) + (m as time_t * 60) + s as time_t
}
// https://github.com/protocolbuffers/upb/blob/22182e6e/upb/json_decode.c#L982
fn epoch_days_fast(y: i32, m: u8, d: u8) -> time_t {
    let (y, m, d) = (y as time_t, m as time_t, d as time_t);
    let year_base = 4800;    /* Before min year, multiple of 400. */
    let m_adj = m - 3;       /* March-based month. */
    // Original code relied on underflow here, but I had to fix it to use signed integers
    let carry = (m_adj < 0) as i64;
    let adjust = if carry == 1 { 12 } else { 0 };
    let y_adj = y + year_base - carry;
    let month_days = ((m_adj + adjust) * 62719 + 769) / 2048;
    let leap_days = y_adj / 4 - y_adj / 100 + y_adj / 400;
    y_adj * 365 + leap_days + month_days + (d - 1) - 2472632
}
