use core::iter::FusedIterator;
use core::ops::{Index, IndexMut};

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::collections::vec_deque::{Iter as DequeIter, Drain as DequeDrain};

use super::{Mutex, MutexGuard, Condvar};

#[derive(Default, Debug)]
pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    read_cond: Condvar,
    write_cond: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        Channel {
            queue: Mutex::new(VecDeque::new()),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Channel<T> {
        Channel {
            queue: Mutex::new(VecDeque::with_capacity(cap)),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
        }
    }

    pub fn push(&self, item: T) {
        self.queue.lock().push_back(item);
        // We need notify_all because it is possible to wait for an item without consuming it, via Channel::wait() or Channel::peek()
        self.read_cond.notify_all();
    }

    pub fn push_bounded(&self, item: T, bound: usize) {
        let mut queue = self.queue.lock();
        loop {
            if queue.len() <= bound {
                queue.push_back(item);
                self.read_cond.notify_all();
                return;
            }
            queue = self.write_cond.wait(queue);
        }
    }

    pub fn recv(&self) -> T {
        let mut queue = self.queue.lock();
        loop {
            if let Some(ret) = queue.pop_front() {
                self.write_cond.notify_one();
                return ret;
            }
            queue = self.read_cond.wait(queue);
        }
    }

    pub fn try_recv(&self) -> Option<T> {
        if let Some(ret) = self.queue.lock().pop_front() {
            self.write_cond.notify_one();
            return Some(ret);
        } else {
            return None;
        }
    }

    pub fn wait(&self) {
        let mut queue = self.queue.lock();
        loop {
            if queue.len() > 0 {
                return;
            }
            queue = self.read_cond.wait(queue);
        }
    }

    pub fn peek(&self) -> T
    where
        T: Clone
    {
        let mut queue = self.queue.lock();
        loop {
            if let Some(ret) = queue.get(0) {
                return ret.clone();
            }
            queue = self.read_cond.wait(queue);
        }
    }

    pub fn try_peek(&self) -> Option<T>
    where
        T: Clone
    {
        self.queue.lock().get(0).cloned()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            channel: self
        }
    }

    pub fn lock(&self) -> LockedChannel<'_, T> {
        LockedChannel {
            queue: self.queue.lock(),
            read_cond: &self.read_cond,
            write_cond: &self.write_cond,
            should_notify_read: false,
            should_notify_write: false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, T> {
    channel: &'a Channel<T>
}

impl<T> Iterator for Iter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.channel.recv())
    }
}

// FusedIterator impl is OK because our Iter never returns None
impl<T> FusedIterator for Iter<'_, T> {}

#[derive(Debug)]
pub struct LockedChannel<'a, T> {
    queue: MutexGuard<'a, VecDeque<T>>,
    read_cond: &'a Condvar,
    write_cond: &'a Condvar,
    should_notify_read: bool,
    should_notify_write: bool,
}

impl<T> LockedChannel<'_, T> {
    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(ret) = self.queue.pop_front() {
            self.should_notify_write = true;
            return Some(ret);
        }
        None
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(ret) = self.queue.pop_front() {
            self.should_notify_write = true;
            return Some(ret);
        }
        None
    }

    pub fn push_front(&mut self, item: T) {
        self.queue.push_front(item);
        self.should_notify_read = true;
    }

    pub fn push_back(&mut self, item: T) {
        self.queue.push_front(item);
        self.should_notify_read = true;
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn peek_iter(&mut self) -> PeekIter<'_, T> {
        PeekIter {
            iter: self.queue.iter(),
        }
    }

    pub fn iter(&mut self) -> LockedIter<'_, T> {
        LockedIter {
            iter: self.queue.drain(..),
            should_notify_write: &mut self.should_notify_write,
        }
    }
}

impl<T> Index<usize> for LockedChannel<'_, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.queue[index]
    }
}

impl<T> IndexMut<usize> for LockedChannel<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.queue[index]
    }
}

impl<T> Drop for LockedChannel<'_, T> {
    fn drop(&mut self) {
        if self.should_notify_read {
            self.read_cond.notify_all();
        }
        if self.should_notify_write {
            self.write_cond.notify_all();
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeekIter<'a, T> {
    iter: DequeIter<'a, T>,
}

impl<'a, T> Iterator for PeekIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn last(self) -> Option<&'a T> {
        self.iter.last()
    }
}

impl<T> ExactSizeIterator for PeekIter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for PeekIter<'_, T> {}

#[derive(Debug)]
pub struct LockedIter<'a, T> {
    iter: DequeDrain<'a, T>,
    should_notify_write: &'a mut bool,
}

impl<T> Iterator for LockedIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let ret = self.iter.next();
        if ret.is_some() { *self.should_notify_write = true; }
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        let ret = self.iter.count();
        if ret > 0 { *self.should_notify_write = true; }
        ret
    }

    fn last(self) -> Option<T> {
        let ret = self.iter.last();
        if ret.is_some() { *self.should_notify_write = true; }
        ret
    }
}

impl<T> ExactSizeIterator for LockedIter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for LockedIter<'_, T> {}
