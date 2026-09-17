// This is the fallback hasher from ahash, vendored as default hasher implementation
// https://github.com/tkaitchuck/ahash

use core::hash::Hasher;

const MULTIPLE: u64 = 6364136223846793005;
const ROT: u32 = 23;

#[inline(always)]
fn folded_multiply(s: u64, by: u64) -> u64 {
    let result = (s as u128).wrapping_mul(by as u128);
    ((result & 0xffff_ffff_ffff_ffff) as u64) ^ ((result >> 64) as u64)
}

#[inline(always)]
#[allow(clippy::len_zero)]
pub(crate) fn read_small(data: &[u8]) -> [u64; 2] {
    debug_assert!(data.len() <= 8);
    if data.len() >= 2 {
        if data.len() >= 4 {
            //len 4-8
            [data.read_u32().0 as u64, data.read_last_u32() as u64]
        } else {
            //len 2-3
            [data.read_u16().0 as u64, data[data.len() - 1] as u64]
        }
    } else {
        if data.len() > 0 {
            [data[0] as u64, data[0] as u64]
        } else {
            [0, 0]
        }
    }
}

#[derive(Debug, Clone)]
pub struct AHasher {
    buffer: u64,
    pad: u64,
    extra_keys: [u64; 2],
}

impl AHasher {
    pub(crate) fn new(k0: u64, k1: u64, k2: u64, k3: u64) -> AHasher {
        AHasher {
            buffer: k0,
            pad: k1,
            extra_keys: [k2, k3],
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_keys(key1: u128, key2: u128) -> AHasher {
        const PI_U128X2: [u128; 2] = [
            0x1319_8a2e_0370_7344_243f_6a88_85a3_08d3,
            0x082e_fa98_ec4e_6c89_a409_3822_299f_31d0,
        ];

        let key1: [u64; 2] = (key1 ^ PI_U128X2[0]).convert();
        let key2: [u64; 2] = (key2 ^ PI_U128X2[1]).convert();
        AHasher {
            buffer: key1[0],
            pad: key1[1],
            extra_keys: key2,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_with_keys(key1: u128, key2: u128) -> Self {
        let key1: [u64; 2] = key1.convert();
        let key2: [u64; 2] = key2.convert();
        Self {
            buffer: key1[0],
            pad: key1[1],
            extra_keys: key2,
        }
    }

    #[inline(always)]
    fn update(&mut self, new_data: u64) {
        self.buffer = folded_multiply(new_data ^ self.buffer, MULTIPLE);
    }

    #[inline(always)]
    fn large_update(&mut self, new_data: u128) {
        let block: [u64; 2] = new_data.convert();
        let combined = folded_multiply(block[0] ^ self.extra_keys[0], block[1] ^ self.extra_keys[1]);
        self.buffer = (self.buffer.wrapping_add(self.pad) ^ combined).rotate_left(ROT);
    }
}

impl Hasher for AHasher {
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.update(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.update(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.update(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.update(i);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.large_update(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }

    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write(&mut self, input: &[u8]) {
        let mut data = input;
        let length = data.len() as u64;
        //Needs to be an add rather than an xor because otherwise it could be canceled with carefully formed input.
        self.buffer = self.buffer.wrapping_add(length).wrapping_mul(MULTIPLE);
        //A 'binary search' on sizes reduces the number of comparisons.
        if data.len() > 8 {
            if data.len() > 16 {
                let tail = data.read_last_u128();
                self.large_update(tail);
                while data.len() > 16 {
                    let (block, rest) = data.read_u128();
                    self.large_update(block);
                    data = rest;
                }
            } else {
                self.large_update([data.read_u64().0, data.read_last_u64()].convert());
            }
        } else {
            let value = read_small(data);
            self.large_update(value.convert());
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        let rot = (self.buffer & 63) as u32;
        folded_multiply(self.buffer, self.pad).rotate_left(rot)
    }
}

trait Convert<To> {
    fn convert(self) -> To;
}

impl Convert<u128> for [u64; 2] {
    #[inline(always)]
    fn convert(self) -> u128 {
        let mut bytes = [0; 16];
        bytes[0..8].copy_from_slice(&self[0].to_ne_bytes());
        bytes[8..16].copy_from_slice(&self[1].to_ne_bytes());
        u128::from_ne_bytes(bytes)
    }
}

impl Convert<[u64; 2]> for u128 {
    #[inline(always)]
    fn convert(self) -> [u64; 2] {
        let bytes = self.to_ne_bytes();
        let a1 = u64::from_ne_bytes(bytes[0..8].try_into().unwrap());
        let a2 = u64::from_ne_bytes(bytes[8..16].try_into().unwrap());
        [a1, a2]
    }
}

trait ReadFromSlice {
    fn read_u16(&self) -> (u16, &[u8]);
    fn read_u32(&self) -> (u32, &[u8]);
    fn read_u64(&self) -> (u64, &[u8]);
    fn read_u128(&self) -> (u128, &[u8]);
    //fn read_last_u16(&self) -> u16;
    fn read_last_u32(&self) -> u32;
    fn read_last_u64(&self) -> u64;
    fn read_last_u128(&self) -> u128;
}

impl ReadFromSlice for [u8] {
    fn read_u16(&self) -> (u16, &[u8]) {
        let (value, rest) = self.split_at(2);
        (u16::from_ne_bytes(value.try_into().unwrap()), rest)
    }

    fn read_u32(&self) -> (u32, &[u8]) {
        let (value, rest) = self.split_at(4);
        (u32::from_ne_bytes(value.try_into().unwrap()), rest)
    }

    fn read_u64(&self) -> (u64, &[u8]) {
        let (value, rest) = self.split_at(8);
        (u64::from_ne_bytes(value.try_into().unwrap()), rest)
    }

    fn read_u128(&self) -> (u128, &[u8]) {
        let (value, rest) = self.split_at(16);
        (u128::from_ne_bytes(value.try_into().unwrap()), rest)
    }

    /*fn read_last_u16(&self) -> u16 {
        let (_, value) = self.split_at(self.len() - 2);
        u16::from_ne_bytes(value.try_into().unwrap())
    }*/

    fn read_last_u32(&self) -> u32 {
        let (_, value) = self.split_at(self.len() - 4);
        u32::from_ne_bytes(value.try_into().unwrap())
    }

    fn read_last_u64(&self) -> u64 {
        let (_, value) = self.split_at(self.len() - 8);
        u64::from_ne_bytes(value.try_into().unwrap())
    }

    fn read_last_u128(&self) -> u128 {
        let (_, value) = self.split_at(self.len() - 16);
        u128::from_ne_bytes(value.try_into().unwrap())
    }
}
