//! # dstd prelude
//! Re-exports from [`alloc`](alloc::alloc) and [`dstd`](crate) that match the prelude of original std

mod alloc {
    pub extern crate alloc;

    // This is to import the vec macro without importing the module. Example taken from std...
    #[allow(hidden_glob_reexports)]
    mod vec {}

    pub use alloc::*;

    // Since alloc::vec module is shadowed, we have to reimport it here
    pub use alloc::vec::Vec;
}
pub use alloc::{format, vec};
pub use alloc::Vec;
pub use alloc::string::{String, ToString};
pub use alloc::boxed::Box;

pub use crate::{print, println, eprint, eprintln, dbg};
