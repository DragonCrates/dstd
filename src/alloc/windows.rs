// Based on https://github.com/rust-lang/rust/blob/main/library/std/src/sys/alloc/windows.rs

use core::ptr;
use core::alloc::Layout;

use crate::sys::windows::minwindef::*;

use super::MIN_ALIGN;
use super::realloc_fallback;

unsafe extern "system" {
    /// Retrieves a handle to the default heap of the calling process. This handle can then be used in subsequent calls to the heap functions.
    fn GetProcessHeap() -> HANDLE;
    /// Allocates a block of memory from a heap. The allocated memory is not movable.
    fn HeapAlloc(
        /* [in] */ hHeap: HANDLE,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ dwBytes: SIZE_T
    ) -> LPVOID;
    /// Reallocates a block of memory from a heap. This function enables you to resize a memory block and change other memory block properties. The allocated memory is not movable.
    fn HeapReAlloc(
        /* [in] */ hHeap: HANDLE,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ lpMem: LPVOID,
        /* [in] */ dwBytes: SIZE_T,
    ) -> LPVOID;
    /// Frees a memory block allocated from a heap by the HeapAlloc or HeapReAlloc function.
    fn HeapFree(
        /* [in] */ hHeap: HANDLE,
        /* [in] */ dwFlags: DWORD,
        /* [in] */ lpMem: LPVOID
    ) -> BOOL;
}

const HEAP_ZERO_MEMORY: DWORD = 0x00000008;

unsafe fn process_heap_alloc(size: usize, flags: DWORD) -> *mut u8 {
    let process_heap = unsafe { GetProcessHeap() };
    if process_heap.is_null() {
        return ptr::null_mut();
    }

    unsafe { HeapAlloc(process_heap, flags, size) as *mut u8 }
}

struct Header(*mut u8);

unsafe fn alloc_ex(layout: Layout, zeroed: bool) -> *mut u8 {
    let flags = if zeroed { HEAP_ZERO_MEMORY } else { 0 };
    if layout.align() <= MIN_ALIGN {
        unsafe { process_heap_alloc(layout.size(), flags) }
    } else {
        let total = layout.align() + layout.size();

        let ptr = unsafe { process_heap_alloc(total, flags) };
        if ptr.is_null() {
            return ptr::null_mut();
        }

        let offset = layout.align() - (ptr.addr() & (layout.align() - 1));
        let aligned = unsafe { ptr.add(offset) };

        unsafe {
            let header = (aligned as *mut Header).sub(1);
            *header = Header(ptr);
        }

        aligned
    }
}

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    unsafe { alloc_ex(layout, false) }
}

pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    unsafe { alloc_ex(layout, true) }
}

pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
    let block;
    if layout.align() <= MIN_ALIGN {
        block = ptr;
    } else {
        block = unsafe { (ptr as *mut Header).sub(1).read().0 };
    }

    unsafe { HeapFree(GetProcessHeap(), 0, block as LPVOID) };
}

pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    if layout.align() <= MIN_ALIGN {
        unsafe { HeapReAlloc(GetProcessHeap(), 0, ptr as LPVOID, new_size) as *mut u8 }
    } else {
        // use realloc fallback because it handles aligned values
        unsafe { realloc_fallback(ptr, layout, new_size) }
    }
}
