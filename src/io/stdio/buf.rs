use core::ops::{Deref, DerefMut};

pub struct Buf<const S: usize> {
    buf: [u8; S],
    start: usize,
    len: usize,
}

impl<const S: usize> Buf<S> {
    pub const fn new() -> Buf<S> {
        Buf {
            buf: [0; S],
            start: 0,
            len: 0,
        }
    }

    pub fn resize(&mut self, newlen: usize) {
        debug_assert!(newlen <= S, "tried to resize buffer beyond max length");
        if S - self.start < newlen {
            // Newlen is more than available length, so we should move contents to beginning
            self.buf.copy_within(self.start..self.len, 0);
            self.start = 0;
            self.len = newlen;
        } else {
            self.len = self.start + newlen;
        }
    }

    pub fn consume(&mut self, n: usize) {
        self.start += n;
        debug_assert!(self.start <= self.len, "tried to consume more than buffer len");
        if self.start == self.len { self.clear(); }
    }

    pub fn buflen(&self) -> usize {
        self.len - self.start
    }

    pub fn maxlen(&self) -> usize {
        S
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.len = 0;
    }
}

impl<const S: usize> Deref for Buf<S> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buf[self.start..self.len]
    }
}

impl<const S: usize> DerefMut for Buf<S> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.start..self.len]
    }
}
