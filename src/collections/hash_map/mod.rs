//! A hash map implemented with linear probing and Robin-Hood hashing
//!
//! It does not support setting a [`BuildHasher`], because when using RH map, [it is very easy to accidentally HashDoS yourself](https://github.com/rust-lang/rust/issues/36481)
//!
//! As for now, the underlying algorithm uses [`AHasher`](https://github.com/tkaitchuck/aHash/blob/a9d649d18d6aefeef106a48252dc6708ee7f9e47/src/fallback_hash.rs) (fallback hash) and [`RandomState`](https://github.com/tkaitchuck/aHash/blob/a9d649d18d6aefeef106a48252dc6708ee7f9e47/src/random_state.rs) from aHash. This is just an implementation detail, and may change at any time. Those types are private and aren't exposed here
//!
//! If you need a safer map, consider using [`hashbrown`](https://docs.rs/hashbrown) with [`siphasher`](https://docs.rs/siphasher). An example of using them together is provided at `dstd/examples/custom-randomstate`
//!
//! [`BuildHasher`]: core::hash::BuildHasher

mod ahash;
use ahash::AHasher;
mod random_state;
use random_state::RandomState;

mod map;
pub use map::{HashMap, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
mod entry;
pub use entry::Entry;
