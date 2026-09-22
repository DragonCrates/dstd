use core::hash::{BuildHasher, Hash};

use super::{RandomState, HashMap};

pub struct Entry<'a, K, V, S = RandomState> {
    key: K,
    map: &'a mut HashMap<K, V, S>,
}

impl<K, V, S> Entry<'_, K, V, S> {
    pub(crate) fn new(key: K, map: &mut HashMap<K, V, S>) -> Entry<'_, K, V, S> {
        Entry { key, map }
    }
}

impl<'a, K, V, S> Entry<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.map.get_mut_or_insert(self.key, value)
    }
}
