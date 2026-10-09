//! Time functions

use core::ffi::{c_int, c_long, c_char};
use core::ops::{Add, Sub};

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

/// Monotonic clock, used to measure time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(sys::Instant);

impl Instant {
    /// Returns current time
    pub fn now() -> Instant {
        Instant(sys::Instant::now())
    }

    /// Returns the amount of time elapsed since this instant
    pub fn elapsed(&self) -> TimeDelta {
        Instant::now() - *self
    }

    /// Returns the amount of time elapsed from another instant to this one
    ///
    /// Equivalent: `self - earlier`
    pub fn duration_since(&self, earlier: Instant) -> TimeDelta {
        self.0.duration_since(earlier.0)
    }
}

impl Sub for Instant {
    type Output = TimeDelta;
    fn sub(self, rhs: Instant) -> TimeDelta {
        self.duration_since(rhs)
    }
}

impl Add<TimeDelta> for Instant {
    type Output = Instant;
    fn add(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 + rhs)
    }
}

impl Sub<TimeDelta> for Instant {
    type Output = Instant;
    fn sub(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 - rhs)
    }
}

/// System clock, that represents real world time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    time: TimeDelta
}

impl SystemTime {
    /// Returns current real time
    pub fn now() -> SystemTime {
        SystemTime {
            time: sys::time()
        }
    }

    /// Formats this time, using global timezone
    pub fn to_global(&self) -> Option<FormatTime> {
        let tm = sys::gmtime(self.as_unix())?;
        Some(FormatTime::from_tm(tm))
    }

    /// Formats this time, using local timezone
    pub fn to_local(&self) -> Option<FormatTime> {
        let tm = sys::localtime(self.as_unix())?;
        Some(FormatTime::from_tm(tm))
    }

    /// Returns unix timestamp (`time_t`)
    pub fn as_unix(&self) -> time_t {
        self.time.as_secs()
    }

    /// Constructs `SystemTime` from a unix timestamp
    pub fn from_unix(time: time_t) -> SystemTime {
        SystemTime { time: TimeDelta::from_secs(time) }
    }

    pub fn as_duration(&self) -> TimeDelta {
        self.time
    }

    pub fn from_duration(time: TimeDelta) -> SystemTime {
        SystemTime { time }
    }
}

/// Signed duration type, stores seconds as `i64`. It is very similar to `struct timespec`, and can losslessly represent one
///
/// When using dstd, you should always prefer using it over [`core::time::Duration`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta {
    secs: i64,
    nanos: u32,
}

const NANOS_PER_SEC: u32 = 1_000_000_000;
impl TimeDelta {
    /// Constructs a new `TimeDelta`. If the number of nanoseconds is greater than 1 billion, they will carry into the seconds counter
    pub fn new(secs: i64, nanos: u32) -> TimeDelta {
        if nanos < NANOS_PER_SEC {
            TimeDelta { secs, nanos }
        } else {
            let secs = secs + (nanos / NANOS_PER_SEC) as i64;
            let nanos = nanos % NANOS_PER_SEC;
            TimeDelta { secs, nanos }
        }
    }

    fn new_normalized(secs: i64, nanos: u32) -> TimeDelta {
        TimeDelta { secs, nanos }
    }

    /// Returns the number of seconds
    pub fn as_secs(&self) -> i64 {
        self.secs
    }

    /// Creates a new `TimeDelta` from the specified amount of seconds
    pub fn from_secs(secs: i64) -> TimeDelta {
        TimeDelta { secs, nanos: 0 }
    }

    /// Returns the number of milliseconds
    pub fn as_millis(&self) -> i64 {
        self.secs * 1000 + (self.nanos / 1_000_000) as i64
    }

    /// Creates a new `TimeDelta` from the specified amount of milliseconds
    pub fn from_millis(millis: i64) -> TimeDelta {
        let secs = millis.div_euclid(1000);
        let nanos = millis.rem_euclid(1000) as u32 * 1_000_000;
        TimeDelta { secs, nanos }
    }

    /// Returns the fractional part of this `TimeDelta` in nanoseconds. Valid range - [0; 1_000_000_000)
    pub fn subsec_nanos(&self) -> u32 {
        self.nanos
    }
}

/// Formatted time structure
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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
        let time = TimeDelta::from_secs(hms + ymd - self.tz_offset as i64);
        SystemTime { time }
    }

    #[allow(clippy::unnecessary_cast)]
    fn from_tm(tm: Tm) -> FormatTime {
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

pub(crate) fn sleep(dur: TimeDelta) {
    sys::sleep(dur);
}

// Thanks for inspiration
// https://blog.reverberate.org/2020/05/12/optimizing-date-algorithms.html
fn hms_to_time(h: u8, m: u8, s: u8) -> i64 {
    (h as i64 * 3600) + (m as i64 * 60) + s as i64
}
// https://github.com/protocolbuffers/upb/blob/22182e6e/upb/json_decode.c#L982
fn epoch_days_fast(y: i32, m: u8, d: u8) -> i64 {
    let (y, m, d) = (y as i64, m as i64, d as i64);
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

#[cfg(test)]
mod tests {
    use super::{SystemTime, FormatTime, TimeDelta};

    #[test]
    fn to_global_works() {
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
    }

    #[test]
    fn to_system_works() {
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
        assert_eq!(1767481606, t.to_system().as_unix());

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
        assert_eq!(-1, t.to_system().as_unix());
    }

    #[test]
    fn from_millis_negative() {
        assert_eq!(-1, TimeDelta::from_millis(-1).as_millis());
        assert_eq!(-1500, TimeDelta::from_millis(-1500).as_millis());
        assert_eq!(-1000, TimeDelta::from_millis(-1000).as_millis());
        assert_eq!(1500, TimeDelta::from_millis(1500).as_millis());
        assert_eq!(0, TimeDelta::from_millis(0).as_millis());
    }
}
