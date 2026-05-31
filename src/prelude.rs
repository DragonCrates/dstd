//! # dstd prelude
//! Re-exports from [`alloc`] and [`dstd`](crate) that match the prelude of original std

mod ambiguous_macros_only {
    #![allow(hidden_glob_reexports)]
    extern crate alloc;

    // This is to import the vec macro without importing the module. Example taken from std...
    mod vec {}

    pub use alloc::*;
}
extern crate alloc;
#[doc(no_inline)]
pub use alloc::format;
#[doc(no_inline)]
pub use alloc::vec::Vec;
#[doc(no_inline)]
pub use alloc::string::{String, ToString};
#[doc(no_inline)]
pub use alloc::boxed::Box;
#[doc(no_inline)]
pub use self::ambiguous_macros_only::vec;

#[doc(no_inline)]
pub use crate::{print, println, eprint, eprintln, dbg};
