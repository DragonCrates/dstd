use core::iter::FusedIterator;
use core::ops::{Index, IndexMut};

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::collections::vec_deque::Iter as DequeIter;

use super::{Mutex, MutexGuard, Condvar};

/// A multiple-producer, multiple-consumer (MPMC) FIFO channel.
///
/// It is an unbounded queue guarded by a [`Mutex`], with two [`Condvar`]s for
/// blocking `send`/`recv` operations. Elements are pushed to the back and
/// received from the front, preserving insertion order.
///
/// The channel has no close/drop-termination: blocking operations (`recv`,
/// `wait`, `peek`) wait forever until data arrives, and [`Channel::iter`] is
/// infinite.
#[derive(Debug)]
pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    read_cond: Condvar,
    write_cond: Condvar,
}

impl<T> Channel<T> {
    /// Creates a new, empty channel.
    pub fn new() -> Channel<T> {
        Channel {
            queue: Mutex::new(VecDeque::new()),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
        }
    }

    /// Creates a new, empty channel with the capacity to hold at least `cap` elements without reallocating
    pub fn with_capacity(cap: usize) -> Channel<T> {
        Channel {
            queue: Mutex::new(VecDeque::with_capacity(cap)),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
        }
    }

    /// Appends `item` to the back of the channel
    pub fn send(&self, item: T) {
        self.queue.lock().push_back(item);
        // We need notify_all because it is possible to wait for an item without consuming it, via Channel::wait() or Channel::peek()
        self.read_cond.notify_all();
    }

    /// Appends `item` to the back of the channel, blocking while the queue is
    /// full (holds at least `bound` elements). After returning, the queue
    /// holds at most `bound` elements.
    ///
    /// Example: `channel.send_bounded("item", 1)` ensures it holds at most 1 element, and `channel.push_bounded("item", 0)` simply blocks forever
    pub fn send_bounded(&self, item: T, bound: usize) {
        let mut queue = self.queue.lock();
        loop {
            if queue.len() < bound {
                queue.push_back(item);
                self.read_cond.notify_all();
                return;
            }
            queue = self.write_cond.wait(queue);
        }
    }

    /// Blocks until the channel is non-empty, then removes and returns the oldest element
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

    /// Removes and returns an item from the queue. Returns `None` immediately if the channel is empty
    pub fn try_recv(&self) -> Option<T> {
        let ret = self.queue.lock().pop_front();
        if ret.is_some() {
            self.write_cond.notify_one();
        }
        ret
    }

    /// Blocks until the channel holds at least one element. Does not consume anything
    pub fn wait(&self) {
        let mut queue = self.queue.lock();
        loop {
            if !queue.is_empty() {
                return;
            }
            queue = self.read_cond.wait(queue);
        }
    }

    /// Blocks until the channel is non-empty, then returns a clone of the first element without consuming it
    pub fn peek(&self) -> T
    where
        T: Clone
    {
        let mut queue = self.queue.lock();
        loop {
            if let Some(ret) = queue.front() {
                return ret.clone();
            }
            queue = self.read_cond.wait(queue);
        }
    }

    /// Returns a clone of the oldest element without consuming it, or `None` if the channel is empty
    pub fn try_peek(&self) -> Option<T>
    where
        T: Clone
    {
        self.queue.lock().front().cloned()
    }

    /// Returns an infinite iterator over elements received from the channel.
    ///
    /// Each call to `next` blocks until an element is available; it never
    /// returns `None`.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            channel: self
        }
    }

    /// Locks the channel so multiple operations can be performed without releasing the mutex in between
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

impl<T> Default for Channel<T> {
    fn default() -> Channel<T> {
        Channel::new()
    }
}

/// An infinite iterator over the elements of a [`Channel`], received by
/// [`Channel::iter`].
///
/// Each call to [`Iterator::next`] blocks until an element is available and
/// never returns `None`.
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

/// A locked view of a [`Channel`], returned by [`Channel::lock`].
///
/// Provides the same operations as [`Channel`], but holds the internal mutex
/// for the whole lifetime of the handle, allowing a batch of operations to be
/// performed atomically. Blocked waiters are only notified when the handle is
/// dropped, after operations that have modified the queue.
#[derive(Debug)]
pub struct LockedChannel<'a, T> {
    queue: MutexGuard<'a, VecDeque<T>>,
    read_cond: &'a Condvar,
    write_cond: &'a Condvar,
    should_notify_read: bool,
    should_notify_write: bool,
}

impl<T> LockedChannel<'_, T> {
    /// Removes and returns the oldest element. Returns `None` if the channel is empty
    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(ret) = self.queue.pop_front() {
            self.should_notify_write = true;
            return Some(ret);
        }
        None
    }

    /// Removes and returns the newest element. Returns `None` if the channel is empty
    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(ret) = self.queue.pop_back() {
            self.should_notify_write = true;
            return Some(ret);
        }
        None
    }

    /// Appends `item` to the front of the channel
    pub fn push_front(&mut self, item: T) {
        self.queue.push_front(item);
        self.should_notify_read = true;
    }

    /// Appends `item` to the back of the channel
    pub fn push_back(&mut self, item: T) {
        self.queue.push_back(item);
        self.should_notify_read = true;
    }

    /// Returns the number of elements currently in the channel
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the elements without consuming them
    pub fn peek_iter(&mut self) -> PeekIter<'_, T> {
        PeekIter {
            iter: self.queue.iter(),
        }
    }

    /// Removes and returns elements currently in the channel, one by one
    pub fn iter(&mut self) -> LockedIter<'_, T> {
        LockedIter {
            queue: &mut *self.queue,
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

/// A non-consuming iterator over the elements of a [`LockedChannel`], created
/// by [`LockedChannel::peek_iter`].
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

/// A consuming iterator over the elements of a [`LockedChannel`], created by
/// [`LockedChannel::iter`].
#[derive(Debug)]
pub struct LockedIter<'a, T> {
    queue: &'a mut VecDeque<T>,
    should_notify_write: &'a mut bool,
}

impl<T> Iterator for LockedIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let ret = self.queue.pop_front();
        if ret.is_some() { *self.should_notify_write = true; }
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.queue.len(), Some(self.queue.len()))
    }

    fn count(self) -> usize {
        let ret = self.queue.len();
        if ret > 0 {
            *self.should_notify_write = true;
            self.queue.clear();
        }
        ret
    }

    fn last(self) -> Option<T> {
        let ret = self.queue.pop_back();
        if ret.is_some() {
            *self.should_notify_write = true;
            self.queue.clear();
        }
        ret
    }
}

impl<T> ExactSizeIterator for LockedIter<'_, T> {
    fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<T> FusedIterator for LockedIter<'_, T> {}
