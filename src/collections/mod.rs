//! Collection types
//!
//! Currently, only [`HashMap`] is provided here. If you need other types, consider using [`alloc::collections`]

extern crate alloc;

pub mod hash_map;
pub use hash_map::HashMap;
