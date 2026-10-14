// Based on https://github.com/rust-lang/rust/blob/main/library/std/src/sys/alloc/unix.rs

use core::ptr;
use core::ffi::c_void;
use core::alloc::Layout;

use super::MIN_ALIGN;
use super::realloc_fallback;

mod libc {
    use core::ffi::{c_int, c_void};
    unsafe extern "C" {
        pub fn malloc(size: usize) -> *mut c_void;
        pub fn calloc(n: usize, size: usize) -> *mut c_void;
        pub fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
        pub fn free(ptr: *mut c_void);
        pub fn posix_memalign(memptr: *mut *mut c_void, alignment: usize, size: usize) -> c_int;
    }
}

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
        unsafe { libc::malloc(layout.size()) as *mut u8 }
    } else {
        unsafe { aligned_alloc(layout) }
    }
}

pub unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
    unsafe { libc::free(ptr as *mut c_void); }
}

pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
        unsafe { libc::calloc(layout.size(), 1) as *mut u8 }
    } else {
        let ptr = unsafe { aligned_alloc(layout) };
        if !ptr.is_null() {
            unsafe { ptr::write_bytes(ptr, 0, layout.size()); }
        }
        ptr
    }
}

pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
        unsafe { libc::realloc(ptr as *mut c_void, new_size) as *mut u8 }
    } else {
        unsafe { realloc_fallback(ptr, layout, new_size) }
    }
}

unsafe fn aligned_alloc(layout: Layout) -> *mut u8 {
    let mut out = ptr::null_mut();
    let align = layout.align().max(size_of::<usize>());
    let ret = unsafe { libc::posix_memalign(&mut out, align, layout.size()) };
    if ret != 0 { ptr::null_mut() } else { out as *mut u8 }
}
