use core::ptr;
use core::alloc::{GlobalAlloc, Layout};

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

#[cfg(any(
    target_arch = "x86",
    target_arch = "arm",
))]
const MIN_ALIGN: usize = 8;
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
))]
const MIN_ALIGN: usize = 16;

pub struct System;

unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { sys::alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { sys::dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { sys::alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { sys::realloc(ptr, layout, new_size) }
    }
}

unsafe fn realloc_fallback(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
    unsafe {
        let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());

        let new_ptr = sys::alloc(new_layout);
        if !new_ptr.is_null() {
            let size = usize::min(old_layout.size(), new_size);
            ptr::copy_nonoverlapping(ptr, new_ptr, size);
            sys::dealloc(ptr, old_layout);
        }

        new_ptr
    }
}
