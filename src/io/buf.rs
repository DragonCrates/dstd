use core::ops::{Deref, DerefMut};

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;

pub struct Buf<B> {
    buf: B,
    start: usize,
    end: usize,
}

impl<B: BufTrait> Buf<B> {
    /// Creates a new buf
    pub fn new() -> Buf<B> {
        Buf {
            buf: B::new(),
            start: 0,
            end: 0,
        }
    }

    /// Returns the space after the end of the buffer, to fill it with new data
    pub fn get_fill(&mut self) -> &mut [u8] {
        &mut self.buf.get_mut()[self.end..]
    }

    /// Extends the buffer with newly written data
    pub fn extend(&mut self, amt: usize) {
        self.end += amt;
        debug_assert!(self.end <= self.buf.get().len(), "attempt to extend buffer beyond its end");
    }

    fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    /// Consumes `amt` bytes from the buffer
    pub fn consume(&mut self, amt: usize) {
        self.start += amt;
        debug_assert!(self.start <= self.end, "tried to consume more than buffer len");
        if self.start == self.end { self.reset(); }
    }

    /// Returns the maximum possible capacity for this buffer
    pub fn capacity(&self) -> usize {
        self.buf.get().len()
    }
}

impl Buf<Stack> {
    /// Creates a new buf. Suitable for constant contexts
    pub const fn const_new() -> Buf<Stack> {
        Buf {
            buf: Stack::const_new(),
            start: 0,
            end: 0,
        }
    }
}

impl<B: BufTrait> Deref for Buf<B> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buf.get()[self.start..self.end]
    }
}

impl<B: BufTrait> DerefMut for Buf<B> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buf.get_mut()[self.start..self.end]
    }
}

pub trait BufTrait {
    fn new() -> Self;
    fn get(&self) -> &[u8];
    fn get_mut(&mut self) -> &mut [u8];
}

pub struct Stack([u8; 256]);
impl BufTrait for Stack {
    fn new() -> Self {
        Stack([0; 256])
    }

    fn get(&self) -> &[u8] {
        &self.0
    }

    fn get_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Stack {
    const fn const_new() -> Stack {
        Stack([0; 256])
    }
}

pub struct Heap(Box<[u8; 4096]>);
impl BufTrait for Heap {
    fn new() -> Self {
        let buf = vec![0; 4096].into_boxed_slice().try_into().unwrap();
        Heap(buf)
    }

    fn get(&self) -> &[u8] {
        &*self.0
    }

    fn get_mut(&mut self) -> &mut [u8] {
        &mut *self.0
    }
}
