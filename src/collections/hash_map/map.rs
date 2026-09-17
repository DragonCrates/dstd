use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::hash::{Hash, BuildHasher};
use core::mem;
use core::ops::Index;
use core::slice::{Iter as SliceIter, IterMut as SliceIterMut};

extern crate alloc;
use alloc::vec;
use alloc::vec::{Vec, IntoIter as VecIntoIter};

use super::RandomState;

#[derive(Default)]
pub struct HashMap<K, V> {
    entries: Vec<Option<Entry<K, V>>>,
    len: usize,
    hasher: RandomState,
}

struct Entry<K, V> {
    key: K,
    value: V,
    hash: u64,
}

fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1) == 0)
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

impl<K, V> HashMap<K, V> {
    pub fn new() -> HashMap<K, V> {
        HashMap {
            entries: vec![],
            len: 0,
            hasher: RandomState::new(),
        }
    }

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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        for e in &mut self.entries {
            *e = None;
        }
        self.len = 0;
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    fn should_resize(&self) -> bool {
        // Approximated self.load_factor() > 0.9
        self.len > self.capacity() - self.capacity() / 8
    }

    fn mask(&self) -> usize {
        debug_assert!(is_power_of_two(self.capacity()));
        self.capacity() - 1
    }

    fn desired_pos(&self, hash: u64) -> usize {
        hash as usize & self.mask()
    }

    fn dist(&self, hash: u64, idx: usize) -> usize {
        (idx + self.capacity() - self.desired_pos(hash)) & self.mask()
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq
{
    fn insert_helper(&mut self, key: K, value: V) -> (usize, Option<V>) {
        // Make capacity
        if self.capacity() == 0 {
            // Init
            self.entries.resize_with(32, Default::default);
        } else if self.should_resize() {
            // Rehash
            debug_assert!(self.capacity() >= 32);
            let mut new = HashMap::with_capacity(self.capacity() * 2);
            mem::swap(self, &mut new);
            for (k, v) in new {
                self.insert(k, v);
            }
            // Rehash done
        }

        debug_assert!(self.capacity() >= 32);

        let cap = self.capacity();
        let mask = cap - 1;
        let hash = self.hasher.hash_one(&key);
        let mut current = Entry { key, value, hash };
        let mut pos = self.desired_pos(hash);
        let mut dist = 0;
        loop {
            let e = &mut self.entries[pos];
            if e.is_none() {
                // Found an empty slot
                *e = Some(current);
                // Successful insertion
                self.len += 1;
                return (pos, None);
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
                    dist = exist_dist;
                }
            }

            pos = (pos + 1) & self.mask();
            dist += 1;
        }
    }

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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let pos = self.get_helper(key)?;
        Some(&self.entries[pos].as_ref().unwrap().value)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

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

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let pos = self.get_helper(key)?;
        Some(self.remove_helper(pos))
    }

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
    pub fn entry(&mut self, key: K) -> super::Entry<'_, K, V> {
        super::Entry::new(key, self)
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.entries.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.entries.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.entries.iter(),
        }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.entries.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.entries.iter_mut(),
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            iter: self.entries.into_iter()
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

pub struct Iter<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some((&e.key, &e.value));
            }
        }
    }
}

pub struct IterMut<'a, K, V> {
    iter: SliceIterMut<'a, Option<Entry<K, V>>>
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some((&e.key, &mut e.value));
            }
        }
    }
}

pub struct Keys<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some(&e.key);
            }
        }
    }
}

pub struct Values<'a, K, V> {
    iter: SliceIter<'a, Option<Entry<K, V>>>
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some(&e.value);
            }
        }
    }
}

pub struct ValuesMut<'a, K, V> {
    iter: SliceIterMut<'a, Option<Entry<K, V>>>
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<&'a mut V> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some(&mut e.value);
            }
        }
    }
}

pub struct IntoIter<K, V> {
    iter: VecIntoIter<Option<Entry<K, V>>>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        loop {
            if let Some(e) = self.iter.next()? {
                return Some((e.key, e.value));
            }
        }
    }
}
