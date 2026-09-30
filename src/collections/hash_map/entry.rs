use core::hash::Hash;

use super::HashMap;

/// A borrowed `HashMap` entry
pub struct Entry<'a, K, V> {
    key: K,
    map: &'a mut HashMap<K, V>,
}

impl<K, V> Entry<'_, K, V> {
    pub(crate) fn new(key: K, map: &mut HashMap<K, V>) -> Entry<'_, K, V> {
        Entry { key, map }
    }
}

impl<'a, K: Hash + Eq, V> Entry<'a, K, V> {
    /// Retrieves the element, or inserts `value` if it was not found
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.map.get_mut_or_insert(self.key, value)
    }
}
