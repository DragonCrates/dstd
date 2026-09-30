#![no_std]
#![no_main]

use dstd::prelude::*;
use core::hash::{Hasher, BuildHasher};
use core::cell::Cell;
use dstd::rand::RandomDevice;
use dstd::io::Read;

use siphasher::sip::SipHasher13;
use hashbrown::HashMap;

#[derive(Clone)]
pub struct RandomState {
    k0: u64,
    k1: u64,
}

fn hashmap_random_keys() -> (u64, u64) {
    let mut bytes = [0; 16];
    match RandomDevice.read(&mut bytes) {
        Ok(16) => {},
        Ok(_) => panic!("RandomDevice returned less bytes than requested"),
        Err(e) => panic!("failed to initialize RandomState: {e}"),
    }

    (u64::from_ne_bytes(bytes[0..8].try_into().unwrap()), u64::from_ne_bytes(bytes[8..16].try_into().unwrap()))
}

impl RandomState {
    pub fn new() -> RandomState {
        thread_local! {
            /// Thread-local indeterminism value, initialized with OS rng
            static INDETERMINISM: Cell<(u64, u64)> = Cell::new(hashmap_random_keys());
        }

        let stack: u64 = 0;
        let sp = &stack as *const u64 as u64;

        // Add stack pointer to make it even less predictable
        let (k0, k1) = INDETERMINISM.get();
        let k0 = k0.wrapping_add(sp);
        let k1 = k1.wrapping_add(sp);

        let mut hasher = SipHasher13::new_with_keys(k0, k1);
        hasher.write_u64(k0);
        let new_k0 = hasher.finish();
        hasher = SipHasher13::new_with_keys(k0, k1);
        hasher.write_u64(k1);
        let new_k1 = hasher.finish();

        INDETERMINISM.set((new_k0, new_k1));
        RandomState {
            k0: new_k0,
            k1: new_k1,
        }
    }
}

impl Default for RandomState {
    fn default() -> RandomState {
        RandomState::new()
    }
}

impl BuildHasher for RandomState {
    type Hasher = SipHasher13;
    fn build_hasher(&self) -> SipHasher13 {
        SipHasher13::new_with_keys(self.k0, self.k1)
    }
}

pub type SipHashMap<K, V> = HashMap<K, V, RandomState>;

mod seal {
    use super::SipHashMap;
    pub trait Sealed {}
    impl<K, V> Sealed for SipHashMap<K, V> {}
}

pub trait HashMapExt: seal::Sealed {
    fn new() -> Self;
    fn with_capacity(cap: usize) -> Self;
}

impl<K, V> HashMapExt for SipHashMap<K, V> {
    fn new() -> SipHashMap<K, V> {
        HashMap::with_hasher(RandomState::new())
    }

    fn with_capacity(cap: usize) -> SipHashMap<K, V> {
        HashMap::with_capacity_and_hasher(cap, RandomState::new())
    }
}

dstd::main!(main);
fn main() {
    let mut map1 = SipHashMap::new();
    map1.insert("Key 1", "Value 1");
    map1.insert("Key 2", "Value 2");
    map1.insert("Key 3", "Value 3");
    println!("Map 1 contents: {map1:?}");
    let mut map2 = SipHashMap::new();
    map2.insert("Key 1", "Value 1");
    map2.insert("Key 2", "Value 2");
    map2.insert("Key 3", "Value 3");
    println!("Map 2 contents: {map2:?}");
}
