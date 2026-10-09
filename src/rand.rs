#[cfg(unix)]
use core::ffi::c_uint;

#[cfg(windows)]
use crate::sys::windows::minwindef::*;
#[cfg(unix)]
use crate::sys::libc::{c_size_t, c_ssize_t};
use crate::io::{self, Read};
#[cfg(unix)]
use crate::io::Error;

// TODO: randint, choice. Needs xoshiro and thread locals support
// split into windows.rs and unix.rs

#[cfg(windows)]
unsafe extern "C" {
    /// The RtlGenRandom function generates a pseudo-random number
    #[link_name = "SystemFunction036"]
    fn RtlGenRandom(
        /* [out] */ RandomBuffer: PVOID,
        /* [in] */ RandomBufferLength: ULONG
    ) -> BOOL;
}

#[cfg(unix)]
unsafe extern "C" {
    fn getrandom(buf: *mut u8, size: c_size_t, flags: c_uint) -> c_ssize_t;
}

pub struct RandomDevice;

impl Read for RandomDevice {
    #[cfg(unix)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { getrandom(buf.as_mut_ptr(), buf.len(), 0) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    #[cfg(windows)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { RtlGenRandom(buf.as_mut_ptr() as PVOID, buf.len() as ULONG) };
        // TODO: replace with an io::Error
        if ret == 0 { panic!("RtlGenRandom failed"); }
        Ok(buf.len())
    }
}
