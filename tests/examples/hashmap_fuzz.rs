use std::collections::HashMap as StdMap;
use dstd::collections::HashMap as DstdMap;

fn main() {
    let seed_args: Vec<String> = std::env::args().skip(1).collect();
    let seeds: Vec<u64> = seed_args.iter().filter_map(|s| s.parse().ok()).collect();
    let seeds: Vec<u64> = if seeds.is_empty() {
        [1, 2, 3, 4, 5, 6].to_vec()
    } else {
        seeds
    };
    for seed in &seeds {
        println!("Seed {seed}");
        fuzz(*seed);
        fuzz_strings(*seed);
        fuzz_retain_clone(*seed);
    }
    println!("OK ({} seeds)", seeds.len());
}

fn fuzz_retain_clone(seed: u64) {
    let mut rng: XorShift = XorShift::new(seed ^ 0xabcdef123456789);
    for round in 0..20 {
        let mut ours: DstdMap<i64, i64> = DstdMap::with_capacity(3);
        let mut reference: StdMap<i64, i64> = StdMap::new();
        for _ in 0..(100 + rng.next() % 500) {
            let k = rng.signed() % 500;
            ours.insert(k, k);
            reference.insert(k, k);
        }

        // retain the same predicate in both
        let m = 2 + (rng.next() % 9) as i64;
        ours.retain(|&k, _| k % m != 0);
        reference.retain(|&k, _| k % m != 0);
        assert_eq!(ours.len(), reference.len(), "retain len mismatch round {round}");

        // full-content compare
        let mut ap = ours.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        ap.sort();
        let mut bp = reference.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        bp.sort();
        assert_eq!(ap, bp, "retain content mismatch round {round}");

        // clone must be equal too
        let cloned = ours.clone();
        let mut cp = cloned.iter().map(|(a, b)| (*a, *b)).collect::<Vec<_>>();
        cp.sort();
        assert_eq!(cp, ap, "clone content mismatch round {round}");
        let _ = &mut ours;
    }
}

fn fuzz(seed: u64) {
    let mut rng: XorShift = XorShift::new(seed);
    let mut ours: DstdMap<i64, i64> = DstdMap::new();
    let mut reference: StdMap<i64, i64> = StdMap::new();

    for step in 0..600_000u64 {
        let op = rng.next() % 100;

        match op {
            0..=34 => {
                let k = rng.signed();
                let v = rng.signed();
                let a = ours.insert(k, v);
                let b = reference.insert(k, v);
                if a != b {
                    panic!("insert mismatch at step {step}: k={k} ours={a:?} std={b:?}");
                }
            }
            35..=54 => {
                let k = rng.signed();
                let a = ours.remove(&k);
                let b = reference.remove(&k);
                if a != b {
                    panic!("remove mismatch at step {step}: k={k} ours={a:?} std={b:?}");
                }
            }
            55..=74 => {
                let k = rng.signed();
                let a = ours.get(&k).copied();
                let b = reference.get(&k).copied();
                if a != b {
                    panic!("get mismatch at step {step}: k={k} ours={a:?} std={b:?}");
                }
                let ca = ours.contains_key(&k);
                let cb = reference.contains_key(&k);
                if ca != cb {
                    panic!("contains mismatch at step {step}: k={k} ours={ca} std={cb}");
                }
            }
            75..=84 => {
                let k = rng.signed();
                let v = rng.signed();
                let a = *ours.entry(k).or_insert(v);
                let b = *reference.entry(k).or_insert(v);
                if a != b {
                    panic!("entry mismatch at step {step}: k={k} ours={a} std={b}");
                }
            }
            85..=89 => {
                let k = rng.signed();
                if let Some(v) = reference.get_mut(&k) {
                    *v += 1;
                }
                if let Some(v) = ours.get_mut(&k) {
                    *v += 1;
                }
            }
            90..=94 => {
                // write through or_insert reference (classic counting pattern)
                let k = rng.signed();
                *ours.entry(k).or_insert(0) += 1;
                *reference.entry(k).or_insert(0) += 1;
            }
            95..=99 => {
                // no-op; final full-content comparison runs after the loop
            }
            _ => unreachable!(),
        }

        if ours.len() != reference.len() {
            panic!("len mismatch at step {step}: ours={} std={}", ours.len(), reference.len());
        }
    }

    let mut vec_pairs: Vec<(i64, i64)> = ours.iter().map(|(k, v)| (*k, *v)).collect();
    vec_pairs.sort();
    let mut ref_pairs: Vec<(i64, i64)> = reference.iter().map(|(k, v)| (*k, *v)).collect();
    ref_pairs.sort();
    assert_eq!(vec_pairs, ref_pairs, "final content mismatch seed {seed}");
}

struct XorShift(u64);
impl XorShift {
    fn new(seed: u64) -> Self {
        XorShift(seed.max(1))
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn signed(&mut self) -> i64 {
        self.next() as i64
    }
    fn string(&mut self) -> String {
        let n = (self.next() % 24) as usize;
        let mut s = String::with_capacity(n);
        for _ in 0..n {
            s.push((b'a' + (self.next() % 26) as u8) as char);
        }
        s
    }
}

fn fuzz_strings(seed: u64) {
    let mut rng: XorShift = XorShift::new(seed ^ 0x9e3779b97f4a7c15);
    let mut ours: DstdMap<String, i64> = DstdMap::new();
    let mut reference: StdMap<String, i64> = StdMap::new();

    for step in 0..150_00u64 {
        let op = rng.next() % 100;
        match op {
            0..=39 => {
                let k = rng.string();
                let v = rng.signed();
                if ours.insert(k.clone(), v) != reference.insert(k, v) {
                    panic!("str insert mismatch at step {step}");
                }
            }
            40..=64 => {
                let k = rng.string();
                if ours.remove(&k) != reference.remove(&k) {
                    panic!("str remove mismatch at step {step}");
                }
            }
            65..=84 => {
                let k = rng.string();
                if ours.get(&k) != reference.get(&k) {
                    panic!("str get mismatch at step {step}: k={k:?}");
                }
            }
            85..=94 => {
                let k = rng.string();
                let v = rng.signed();
                let a = *ours.entry(k.clone()).or_insert(v);
                let b = *reference.entry(k).or_insert(v);
                if a != b {
                    panic!("str entry mismatch at step {step}");
                }
            }
            95..=99 => {
                for (k, v) in reference.iter() {
                    if ours.get(k) != Some(v) {
                        panic!("str coherence mismatch at step {step}: k={k:?}");
                    }
                }
            }
            _ => unreachable!(),
        }
        if ours.len() != reference.len() {
            panic!("str len mismatch at step {step}: ours={} std={}", ours.len(), reference.len());
        }
    }
}
