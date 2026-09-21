use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::hash::{Hash, BuildHasher};
use core::iter::FusedIterator;
use core::mem;
use core::ops::Index;
use core::slice::{Iter as SliceIter, IterMut as SliceIterMut};

extern crate alloc;
use alloc::vec;
use alloc::vec::{Vec, IntoIter as VecIntoIter};

use super::RandomState;

/// A hash map implemented with linear probing and Robin-Hood hashing
#[derive(Default)]
pub struct HashMap<K, V> {
    entries: Vec<Option<Entry<K, V>>>,
    len: usize,
    hasher: RandomState,
}

#[derive(Clone)]
struct Entry<K, V> {
    key: K,
    value: V,
    hash: u64,
}

fn calculate_capacity(cap: usize) -> usize {
    if cap == 0 {
        0
    } else if cap < 32 {
        32
    } else {
        usize::next_power_of_two(cap)
    }
}

fn load_capacity(cap: usize) -> usize {
    // Approximated 0.9
    cap - cap / 8
}

impl<K, V> HashMap<K, V> {
    /// Creates an empty `HashMap`.
    ///
    /// The map is initially allocated with no capacity and grows on the first
    /// insertion.
    pub fn new() -> HashMap<K, V> {
        HashMap {
            entries: vec![],
            len: 0,
            hasher: RandomState::new(),
        }
    }

    /// Creates an empty `HashMap` with at least the specified capacity.
    ///
    /// The map will be able to hold at least `cap` elements without
    /// reallocating. The internal table size is rounded up to a power of two,
    /// with a minimum of 32 slots.
    pub fn with_capacity(cap: usize) -> HashMap<K, V> {
        let cap = calculate_capacity(cap);
        let mut entries = vec![];
        entries.resize_with(cap, Default::default);
        HashMap {
            entries,
            len: 0,
            hasher: RandomState::new(),
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated
    /// capacity for reuse.
    pub fn clear(&mut self) {
        for e in &mut self.entries {
            *e = None;
        }
        self.len = 0;
    }

    /// Returns the number of elements the map can hold without reallocating. Actual count is slightly lower, because it reallocates once hitting max load factor
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Amount of elements this map can hold taking max load factor into account
    fn load_capacity(&self) -> usize {
        load_capacity(self.capacity())
    }

    fn should_resize(&self) -> bool {
        self.len > self.load_capacity()
    }

    fn mask(&self) -> usize {
        debug_assert!(self.capacity().is_power_of_two());
        self.capacity() - 1
    }

    fn desired_pos(&self, hash: u64) -> usize {
        hash as usize & self.mask()
    }

    fn dist(&self, hash: u64, idx: usize) -> usize {
        (idx + self.capacity() - self.desired_pos(hash)) & self.mask()
    }
}

fn resize_capacity(cap: usize) -> usize {
    let mut new_size = calculate_capacity(cap);
    if cap > load_capacity(new_size) {
        new_size *= 2;
    }
    new_size
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq
{
    fn rehash_helper(&mut self, new_size: usize) {
        // We have to construct a new RandomState on each rehash - otherwise they would become quadratic
        let new = HashMap::with_capacity(new_size);
        let old = mem::replace(self, new);
        debug_assert!(old.len <= self.load_capacity(), "new map can't hold current amount of elements");
        for (k, v) in old {
            self.insert(k, v);
        }
    }

    /// Reserves space for `n` elements
    pub fn reserve(&mut self, n: usize) {
        let new_size = resize_capacity(self.len + n);
        if new_size != self.capacity() {
            self.rehash_helper(new_size);
        }
    }

    /// Shrinks capacity to `n` (can't shrink below `self.len()`)
    pub fn shrink_to(&mut self, n: usize) {
        let new_size = resize_capacity(self.len.max(n));
        if new_size != self.capacity() {
            self.rehash_helper(new_size);
        }
    }

    /// Shrinks the map to lowest possible capacity
    pub fn shrink_to_fit(&mut self) {
        let new_size = resize_capacity(self.len);
        if new_size != self.capacity() {
            self.rehash_helper(new_size);
        }
    }

    fn insert_helper(&mut self, key: K, value: V) -> (usize, Option<V>) {
        // Make capacity
        if self.capacity() == 0 {
            // Init
            self.entries.resize_with(32, Default::default);
        } else if self.should_resize() {
            // Rehash
            debug_assert!(self.capacity() >= 32);
            self.rehash_helper(self.capacity() * 2);
        }

        debug_assert!(self.capacity() >= 32);

        let cap = self.capacity();
        let mask = cap - 1;
        let hash = self.hasher.hash_one(&key);
        let mut current = Entry { key, value, hash };
        let mut pos = self.desired_pos(hash);
        let mut dist = 0;
        // Remembered position for the inserted entry
        let mut key_pos = None;
        loop {
            let e = &mut self.entries[pos];
            if e.is_none() {
                // Found an empty slot
                *e = Some(current);
                // Successful insertion
                self.len += 1;
                return (key_pos.unwrap_or(pos), None);
            } else {
                // Occupied slot
                let e = e.as_mut().unwrap();
                // Same key?
                if e.hash == current.hash && e.key == current.key {
                    // Overwrite the entry
                    mem::swap(e, &mut current);
                    return (pos, Some(current.value));
                }
                // Had to inline self.dist(e.hash, pos) here, thanks borrow checker
                let exist_dist = (pos + cap - (e.hash as usize & mask)) & mask;
                // Should swap?
                if exist_dist < dist {
                    mem::swap(e, &mut current);
                    // Remember the position, if this is the first swap...
                    if key_pos.is_none() {
                        key_pos = Some(pos);
                    }
                    dist = exist_dist;
                }
            }

            pos = (pos + 1) & self.mask();
            dist += 1;
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned. If the map
    /// did have this key present, the value is updated and the old value is
    /// returned
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert_helper(key, value).1
    }

    fn get_helper<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.entries.is_empty() || self.is_empty() { return None; }

        let hash = self.hasher.hash_one(key);
        let mut pos = self.desired_pos(hash);
        let mut dist = 0;
        loop {
            let e = &self.entries[pos];

            // Empty slot - terminate search
            if e.is_none() {
                return None;
            }

            // Non-empty slot
            let e = e.as_ref().unwrap();

            // Dist larger - terminate search
            if dist > self.dist(e.hash, pos) {
                return None;
            }

            // Found?
            if e.hash == hash && e.key.borrow() == key {
                return Some(pos);
            }

            pos = (pos + 1) & self.mask();
            dist += 1;
        }
    }

    /// Returns a reference to the value corresponding to the key
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let pos = self.get_helper(key)?;
        Some(&self.entries[pos].as_ref().unwrap().value)
    }

    /// Returns `true` if the map contains a value for the specified key
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Returns a mutable reference to the value corresponding to the key
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let pos = self.get_helper(key)?;
        Some(&mut self.entries[pos].as_mut().unwrap().value)
    }

    pub(crate) fn get_mut_or_insert(&mut self, key: K, value: V) -> &mut V {
        let pos = self.get_helper(&key);
        if let Some(pos) = pos {
            &mut self.entries[pos].as_mut().unwrap().value
        } else {
            let pos = self.insert_helper(key, value).0;
            &mut self.entries[pos].as_mut().unwrap().value
        }
    }

    /// Remove by slot index
    fn remove_helper(&mut self, idx: usize) -> V {
        // Perform backward shift deletion from idx to stop

        // Find the stop slot (empty or dist=0)
        let mut stop = (idx + 1) & self.mask();
        loop {
            let e = &self.entries[stop];
            if e.is_none() { break; }
            let e = e.as_ref().unwrap();
            if self.dist(e.hash, stop) == 0 { break; }
            stop = (stop + 1) & self.mask();
        }

        // Delete element
        let ret = self.entries[idx].take().unwrap();
        self.len -= 1;

        // Shift backwards everything after idx and before stop
        let mut i = idx;
        loop {
            let next = (i + 1) & self.mask();
            if next == stop { break; }
            self.entries.swap(i, next);
            i = next;
        }

        ret.value
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let pos = self.get_helper(key)?;
        Some(self.remove_helper(pos))
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, removes all pairs `(k, v)` for which `f(&k, &mut v)`
    /// returns `false`. The elements are visited in unspecified order.
    /// # Example
    /// ```
    /// use dstd::collections::HashMap;
    ///
    /// let mut map: HashMap<i32, i32> = (0..8).map(|x| (x, x*10)).collect();
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert_eq!(map.len(), 4);
    /// ```
    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut f: F) {
        let mut i = 0;
        while i < self.capacity() {
            let e = &mut self.entries[i];
            if e.is_none() { i += 1; continue; }
            let e = e.as_mut().unwrap();
            if !f(&e.key, &mut e.value) {
                self.remove_helper(i);
                // Check again...
            } else {
                // Next element
                i += 1;
            }
        }
    }
}

impl<K, V> Debug for HashMap<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Clone for HashMap<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    fn clone(&self) -> HashMap<K, V> {
        // We need to create a second map manually because RandomState should not be cloned
        let mut new = HashMap::with_capacity(self.capacity());
        for (k, v) in self {
            new.insert(k.clone(), v.clone());
        }
        new
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for HashMap<K, V> {
    fn eq(&self, other: &HashMap<K, V>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<K: Eq, V: Eq> Eq for HashMap<K, V> {}

impl<K, Q, V> Index<&Q> for HashMap<K, V>
where
    K: Hash + Eq + Borrow<Q>,
    Q: Hash + Eq + ?Sized,
{
    type Output = V;
    fn index(&self, index: &Q) -> &V {
        self.get(index).expect("no entry for such key")
    }
}

// IndexMut is not implemented because this syntax don't work:
// `map[key] = value;`
// due to how rust uses IndexMut, this will always panic
// If we match C++ behavior (insert a default element) then it will be extremely confusing because immutable Index can't do that

impl<K, V> HashMap<K, V> {
    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    ///
    /// Currently only `Entry::or_insert` is available on the returned entry.
    pub fn entry(&mut self, key: K) -> super::Entry<'_, K, V> {
        super::Entry::new(key, self)
    }

    /// An iterator visiting all key-value pairs in unspecified order. The
    /// iterator element type is `(&K, &V)`.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.entries.iter(),
            remain: self.len,
        }
    }

    /// An iterator visiting all key-value pairs in unspecified order, with
    /// mutable references to the values. The iterator element type is
    /// `(&K, &mut V)`.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.entries.iter_mut(),
            remain: self.len,
        }
    }

    /// An iterator visiting all keys in unspecified order. The iterator
    /// element type is `&K`.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.entries.iter(),
            remain: self.len,
        }
    }

    /// An iterator visiting all values in unspecified order. The iterator
    /// element type is `&V`.
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.entries.iter(),
            remain: self.len,
        }
    }

    /// An iterator visiting all values in unspecified order, with mutable
    /// references to the values. The iterator element type is `&mut V`.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.entries.iter_mut(),
            remain: self.len,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut map = HashMap::with_capacity(iter.size_hint().0);
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            iter: self.entries.into_iter(),
            remain: self.len,
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut HashMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

impl<K, V> Extend<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let reserve = iter.size_hint().0;
        self.reserve(reserve);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

/// An iterator over the key-value pairs of a `HashMap`. Created by
/// [`HashMap::iter`].
#[derive(Default, Clone)]
pub struct Iter<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>,
    remain: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some((&e.key, &e.value));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<(&'a K, &'a V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some((&e.key, &e.value));
            }
        }
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.clone().filter_map(|i| i.as_ref()).map(|e| (&e.key, &e.value));
        f.debug_list().entries(iter).finish()
    }
}

/// An iterator over the key-value pairs of a `HashMap`, with mutable values.
/// Created by [`HashMap::iter_mut`].
#[derive(Default)]
pub struct IterMut<'a, K, V> {
    iter: SliceIterMut<'a, Option<Entry<K, V>>>,
    remain: usize,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some((&e.key, &mut e.value));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<(&'a K, &'a mut V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some((&e.key, &mut e.value));
            }
        }
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.as_slice().iter().filter_map(|i| i.as_ref()).map(|e| (&e.key, &e.value));
        f.debug_list().entries(iter).finish()
    }
}

/// An iterator over the keys of a `HashMap`. Created by [`HashMap::keys`].
#[derive(Default, Clone)]
pub struct Keys<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>,
    remain: usize,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some(&e.key);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<&'a K> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some(&e.key);
            }
        }
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.clone().filter_map(|i| i.as_ref()).map(|e| &e.key);
        f.debug_list().entries(iter).finish()
    }
}

/// An iterator over the values of a `HashMap`. Created by [`HashMap::values`].
#[derive(Default, Clone)]
pub struct Values<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>,
    remain: usize,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some(&e.value);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<&'a V> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some(&e.value);
            }
        }
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.clone().filter_map(|i| i.as_ref()).map(|e| &e.value);
        f.debug_list().entries(iter).finish()
    }
}

/// An iterator over the values of a `HashMap`, with mutable references.
/// Created by [`HashMap::values_mut`].
#[derive(Default)]
pub struct ValuesMut<'a, K, V> {
    iter: SliceIterMut<'a, Option<Entry<K, V>>>,
    remain: usize,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<&'a mut V> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some(&mut e.value);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<&'a mut V> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some(&mut e.value);
            }
        }
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

impl<K: Debug, V: Debug> fmt::Debug for ValuesMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.as_slice().iter().filter_map(|i| i.as_ref()).map(|e| &e.value);
        f.debug_list().entries(iter).finish()
    }
}

/// An owning iterator over the key-value pairs of a `HashMap`. Created by
/// [`IntoIterator`] or [`HashMap::into_iter`].
#[derive(Default)]
pub struct IntoIter<K, V> {
    iter: VecIntoIter<Option<Entry<K, V>>>,
    remain: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next()? {
                self.remain -= 1;
                return Some((e.key, e.value));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn count(self) -> usize {
        self.remain
    }

    fn last(mut self) -> Option<(K, V)> {
        if self.remain == 0 { return None; }
        loop {
            if let Some(e) = self.iter.next_back()? {
                return Some((e.key, e.value));
            }
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.remain
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K: Clone, V: Clone> Clone for IntoIter<K, V> {
    fn clone(&self) -> IntoIter<K, V> {
        let new: Vec<_> = self.iter.as_slice().iter().filter(|i| i.is_some()).cloned().collect();
        IntoIter {
            iter: new.into_iter(),
            remain: self.remain,
        }
    }
}

impl<K: Debug, V: Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = self.iter.as_slice().iter().filter_map(|i| i.as_ref()).map(|e| (&e.key, &e.value));
        f.debug_list().entries(iter).finish()
    }
}
