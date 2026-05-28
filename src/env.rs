use core::cell::UnsafeCell;

use crate::prelude::{String, ToString};
use crate::ffi::{OsStr, OsChar};

#[cfg(unix)]
crate::block! {
    unsafe extern "C" {
        pub fn getenv(name: *const u8) -> *mut u8;
        static environ: *const *const u8;
    }

    pub fn getenviron() -> *const *const u8 {
        unsafe { environ }
    }
}
#[cfg(windows)]
crate::block! {
    unsafe extern "C" {
        #[link_name = "_wgetenv"]
        fn getenv(name: *const u16) -> *mut u16;
        fn __p__wenviron() -> *const *const *const u16;
    }

    fn getenviron() -> *const *const u16 {
        unsafe {
            let mut wenviron = *__p__wenviron();
            // wenviron is not initialized in programs that use main instead of wmain
            if wenviron.is_null() {
                // this does not race, getenv is thread safe on windows
                getenv([0].as_ptr());
                wenviron = *__p__wenviron();
            }
            wenviron
        }
    }
}

pub(crate) struct SyncUnsafeCell<T>(pub UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

pub(crate) static ARGS: SyncUnsafeCell<&[*const OsChar]> = SyncUnsafeCell(UnsafeCell::new(&[]));

pub struct Args {
    index: usize,
}

pub fn args() -> Args {
    Args { index: 0 }
}

impl Iterator for Args {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let args = unsafe { *ARGS.0.get() };
        let ptr = args.get(self.index)?;
        self.index += 1;

        let s = unsafe { OsStr::from_ptr(*ptr) };
        Some(s.to_utf8().unwrap())
    }
}

pub struct Vars {
    i: *const *const OsChar,
}

pub fn vars() -> Vars {
    Vars { i: getenviron() }
}

impl Iterator for Vars {
    type Item = (String, String);

    fn next(&mut self) -> Option<(String, String)> {
        let ptr = unsafe { *self.i };
        if ptr.is_null() { return None; }
        self.i = unsafe { self.i.add(1) };

        let os_str = unsafe { OsStr::from_ptr(ptr) };
        let s = os_str.to_utf8().unwrap();
        let eq = s.find('=').unwrap();
        let name = s[..eq].to_string();
        let value = s[eq+1..].to_string();
        Some((name, value))
    }
}

pub fn var(name: &str) -> Option<String> {
    let mut buf = [0; 256];
    let name_os = crate::ffi::str_to_os(name, &mut buf).unwrap();

    let ret = unsafe { getenv(name_os.as_ptr()) };
    if ret.is_null() { return None; }

    let ret_os = unsafe { OsStr::from_ptr(ret) };
    let ret_str = ret_os.to_utf8().unwrap();
    Some(ret_str.to_string())
}
