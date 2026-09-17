use core::hash::Hash;

use super::HashMap;

pub struct Entry<'a, K, V> {
    key: K,
    map: &'a mut HashMap<K, V>,
}

impl<'a, K, V> Entry<'a, K, V> {
    pub(crate) fn new(key: K, map: &mut HashMap<K, V>) -> Entry<'_, K, V> {
        Entry { key, map }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq
{
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.map.get_mut_or_insert(self.key, value)
    }
}
