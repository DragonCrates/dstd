use core::ptr;
use core::time::Duration;
use core::ffi::{c_int, c_long};
use core::ops::{Add, Sub};

use super::{Tm, time_t};
use crate::io::{Error, errno};

#[cfg(any(target_os = "linux", target_os = "android"))]
crate::block! {
    /// Clock ID for the clock and timer functions
    #[allow(non_camel_case_types)]
    type clockid_t = c_int;
    const CLOCK_MONOTONIC: clockid_t = 1;
    const TIMER_ABSTIME: c_int = 0x01;
}

unsafe extern "C" {
    /// Retrieve the time of the specified clock clockid
    fn clock_gettime(clockid: clockid_t, tp: *mut Timespec) -> c_int;
    /// High-resolution sleep with specifiable clock
    fn clock_nanosleep(
        clockid: clockid_t,
        flags: c_int,
        t: *const Timespec,
        remain: *mut Timespec,
    ) -> c_int;
    /// Converts the calendar time timep to broken-down time representation, expressed in Coordinated Universal Time (UTC)
    pub fn gmtime_r(timep: *const time_t, result: *mut Tm) -> *mut Tm;
    /// Converts the calendar time timep to broken-down time representation, expressed relative to the user's specified timezone
    pub fn localtime_r(timep: *const time_t, result: *mut Tm) -> *mut Tm;
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

impl Timespec {
    pub fn from_duration(dur: Duration) -> Timespec {
        Timespec {
            tv_sec: dur.as_secs().try_into().unwrap_or(time_t::MAX),
            tv_nsec: dur.subsec_nanos().into(),
        }
    }

    pub fn to_duration(self) -> Duration {
        Duration::new(self.tv_sec as u64, self.tv_nsec as u32)
    }
}

impl Add for Timespec {
    type Output = Timespec;
    fn add(self, rhs: Timespec) -> Timespec {
        let mut out = Timespec {
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

impl Sub for Timespec {
    type Output = Timespec;
    fn sub(self, rhs: Timespec) -> Timespec {
        let mut out = Timespec {
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

#[derive(Debug, Clone, Copy)]
pub struct Instant(Timespec);

impl Instant {
    pub fn now() -> Instant {
        let mut out = Timespec::default();
        let ret = unsafe { clock_gettime(
            CLOCK_MONOTONIC, // clockid
            &raw mut out, // tp
        ) };
        assert!(ret != -1, "clock_gettime failed: {}", Error::last_os_error());
        Instant(out)
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        (self.0 - earlier.0).to_duration()
    }
}

pub fn gmtime(time: time_t) -> Option<Tm> {
    let mut tm = Tm::default();
    let ret = unsafe { gmtime_r(
        &raw const time, // timep
        &raw mut tm, // result
    ) };
    if ret.is_null() { return None; }
    Some(tm)
}

pub fn localtime(time: time_t) -> Option<Tm> {
    let mut tm = Tm::default();
    let ret = unsafe { localtime_r(
        &raw const time, // timep
        &raw mut tm, // result
    ) };
    if ret.is_null() { return None; }
    Some(tm)
}

pub fn sleep(dur: Duration) {
    let start = Instant::now().0;
    let end = start + Timespec::from_duration(dur);
    loop {
        let res = unsafe { clock_nanosleep(
            CLOCK_MONOTONIC, // clockid
            TIMER_ABSTIME, // flags
            &raw const end, // t
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
