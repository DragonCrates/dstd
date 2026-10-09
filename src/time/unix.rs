use core::ptr;
use core::ffi::{c_int, c_long};
use core::ops::{Add, Sub};

use super::{time_t, Tm, TimeDelta};
use crate::io::Error;
use crate::sys::libc::errno;

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    /// Clock ID for the clock and timer functions
    #[allow(non_camel_case_types)]
    type clockid_t = c_int;
    const CLOCK_REALTIME: clockid_t = 0;
    const CLOCK_MONOTONIC: clockid_t = 1;
    const TIMER_ABSTIME: c_int = 0x01;
}

unsafe extern "C" {
    /// Retrieve the time of the specified clock clockid
    fn clock_gettime(clockid: clockid_t, tp: *mut timespec) -> c_int;
    /// High-resolution sleep with specifiable clock
    fn clock_nanosleep(
        clockid: clockid_t,
        flags: c_int,
        t: *const timespec,
        remain: *mut timespec,
    ) -> c_int;
    /// Converts the calendar time timep to broken-down time representation, expressed in Coordinated Universal Time (UTC)
    pub fn gmtime_r(timep: *const time_t, result: *mut Tm) -> *mut Tm;
    /// Converts the calendar time timep to broken-down time representation, expressed relative to the user's specified timezone
    pub fn localtime_r(timep: *const time_t, result: *mut Tm) -> *mut Tm;
}

// Ord is OK as long as our timespec is normalized
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct timespec {
    tv_sec: time_t,
    tv_nsec: c_long,
}

impl timespec {
    pub fn from_duration(dur: TimeDelta) -> timespec {
        timespec {
            tv_sec: dur.as_secs(),
            tv_nsec: dur.subsec_nanos() as c_long,
        }
    }

    pub fn to_duration(self) -> TimeDelta {
        TimeDelta::new(self.tv_sec, self.tv_nsec as u32)
    }
}

impl Add for timespec {
    type Output = timespec;
    fn add(self, rhs: timespec) -> timespec {
        let mut out = timespec {
            tv_sec: self.tv_sec.saturating_add(rhs.tv_sec),
            tv_nsec: self.tv_nsec + rhs.tv_nsec,
        };
        if out.tv_nsec >= 1_000_000_000 {
            out.tv_sec += 1;
            out.tv_nsec -= 1_000_000_000;
        }
        out
    }
}

impl Sub for timespec {
    type Output = timespec;
    fn sub(self, rhs: timespec) -> timespec {
        let mut out = timespec {
            tv_sec: self.tv_sec.saturating_sub(rhs.tv_sec),
            tv_nsec: self.tv_nsec - rhs.tv_nsec,
        };
        if out.tv_nsec < 0 {
            out.tv_sec -= 1;
            out.tv_nsec += 1_000_000_000;
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(timespec);

impl Instant {
    pub fn now() -> Instant {
        let mut out = timespec::default();
        let ret = unsafe { clock_gettime(
            CLOCK_MONOTONIC, // clockid
            &mut out, // tp
        ) };
        assert!(ret != -1, "clock_gettime failed: {}", Error::last_os_error());
        Instant(out)
    }

    pub fn duration_since(&self, earlier: Instant) -> TimeDelta {
        (self.0 - earlier.0).to_duration()
    }
}

impl Add<TimeDelta> for Instant {
    type Output = Instant;
    fn add(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 + timespec::from_duration(rhs))
    }
}

impl Sub<TimeDelta> for Instant {
    type Output = Instant;
    fn sub(self, rhs: TimeDelta) -> Instant {
        Instant(self.0 - timespec::from_duration(rhs))
    }
}

pub fn gmtime(time: time_t) -> Option<Tm> {
    let mut tm = Tm::default();
    let ret = unsafe { gmtime_r(
        &time, // timep
        &mut tm, // result
    ) };
    if ret.is_null() { return None; }
    Some(tm)
}

pub fn localtime(time: time_t) -> Option<Tm> {
    let mut tm = Tm::default();
    let ret = unsafe { localtime_r(
        &time, // timep
        &mut tm, // result
    ) };
    if ret.is_null() { return None; }
    Some(tm)
}

pub fn sleep(dur: TimeDelta) {
    if dur.as_millis() < 0 {
        // skip sleeping if duration is negative
        if cfg!(debug_assertions) {
            panic!("cannot sleep for a negative duration");
        }
        return;
    }

    let start = Instant::now().0;
    let end = start + timespec::from_duration(dur);
    loop {
        let res = unsafe { clock_nanosleep(
            CLOCK_MONOTONIC, // clockid
            TIMER_ABSTIME, // flags
            &end, // t
            ptr::null_mut(), // remain
        ) };
        if res == 0 {
            break;
        } else if res == errno::EINTR {
            // interrupted by signal
        } else {
            panic!("clock_nanosleep failed: {}", Error::from_raw_os_error(res));
        }
    }
}

pub fn time() -> TimeDelta {
    let mut out = timespec::default();
    let ret = unsafe { clock_gettime(
        CLOCK_REALTIME, // clockid
        &mut out, // tp
    ) };
    assert!(ret != -1, "clock_gettime failed: {}", Error::last_os_error());
    TimeDelta::new_normalized(out.tv_sec, out.tv_nsec as u32)
}
