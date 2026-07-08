use crate::ffi::*;
use crate::io::{self, Error, Read};

// TODO: randint, choice. Needs xoshiro and thread locals support

unsafe extern "C" {
    #[cfg(unix)]
    fn getrandom(buf: *mut u8, size: c_size_t, flags: c_uint) -> c_ssize_t;
}

pub struct RandomDevice(());

impl RandomDevice {
    pub fn new() -> RandomDevice {
        RandomDevice(())
    }
}

impl Read for RandomDevice {
    #[cfg(unix)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { getrandom(buf.as_mut_ptr(), buf.len(), 0) };
        if ret == -1 { return Err(Error::last_os_error()); }
        Ok(ret as usize)
    }

    #[cfg(windows)]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0) // TODO windows
    }
}
