use core::fmt;
use core::hash::{Hasher, BuildHasher};
use core::sync::atomic::{AtomicUsize, Ordering};

use super::AHasher;

use crate::io::Read;
use crate::rand::RandomDevice;
use crate::sync::{Once, OnceLock};

#[inline]
fn get_fixed_seeds() -> &'static [[u64; 4]; 2] {
    static SEEDS: OnceLock<[[u64; 4]; 2]> = OnceLock::new();

    SEEDS.get_or_init(|| {
        let mut result: [u8; 64] = [0; 64];
        let nr = RandomDevice.read(&mut result).expect("RandomDevice::read() failed");
        assert!(nr == result.len());
        [
            [
                u64::from_ne_bytes(result[0..8].try_into().unwrap()),
                u64::from_ne_bytes(result[8..16].try_into().unwrap()),
                u64::from_ne_bytes(result[16..24].try_into().unwrap()),
                u64::from_ne_bytes(result[24..32].try_into().unwrap()),
            ],
            [
                u64::from_ne_bytes(result[32..40].try_into().unwrap()),
                u64::from_ne_bytes(result[40..48].try_into().unwrap()),
                u64::from_ne_bytes(result[48..56].try_into().unwrap()),
                u64::from_ne_bytes(result[56..64].try_into().unwrap()),
            ],
        ]
    })
}

#[inline]
fn gen_hasher_seed() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    // Init counter
    static ONCE: Once = Once::new();
    ONCE.call_once(|| COUNTER.store(&COUNTER as *const _ as usize, Ordering::Relaxed));

    // Add the address of an arbitrary stack value
    let stack: u8 = 0;
    COUNTER.fetch_add(&stack as *const _ as usize, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct RandomState {
    k0: u64,
    k1: u64,
    k2: u64,
    k3: u64,
}

impl RandomState {
    #[inline]
    pub fn new() -> RandomState {
        let fixed = get_fixed_seeds();
        Self::from_keys(&fixed[0], &fixed[1], gen_hasher_seed())
    }

    fn from_keys(a: &[u64; 4], b: &[u64; 4], c: usize) -> RandomState {
        let mut hasher = AHasher::new(a[0], a[1], a[2], a[3]);
        hasher.write_usize(c);
        let mix = |l: u64, r: u64| {
            let mut h = hasher.clone();
            h.write_u64(l);
            h.write_u64(r);
            h.finish()
        };
        RandomState {
            k0: mix(b[0], b[2]),
            k1: mix(b[1], b[3]),
            k2: mix(b[2], b[1]),
            k3: mix(b[3], b[0]),
        }
    }
}

impl BuildHasher for RandomState {
    type Hasher = AHasher;

    fn build_hasher(&self) -> AHasher {
        AHasher::new(self.k0, self.k1, self.k2, self.k3)
    }
}

impl fmt::Debug for RandomState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RandomState").finish_non_exhaustive()
    }
}

impl Default for RandomState {
    fn default() -> RandomState {
        RandomState::new()
    }
}
