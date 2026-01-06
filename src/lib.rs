//! Typed and dropless arena allocation, paraphrased from [the Rust Compiler's `rustc_arena`](https://github.com/rust-lang/rust/blob/master/compiler/rustc_arena/src/lib.rs). See [LICENSE][1].
//!
//! An Arena Allocator is a type of allocator which provides stable locations for allocations within
//! itself for the entire duration of its lifetime.
//!
//! [1]: https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-MIT

#![cfg_attr(feature = "nightly", feature(dropck_eyepatch))]
#![no_std]

extern crate alloc;

pub(crate) mod constants {
    //! Size constants for arena chunk growth
    pub(crate) const MIN_CHUNK: usize = 512;
    pub(crate) const MAX_CHUNK: usize = 2 * 1024 * 1024;
}

mod chunk;

#[cfg(feature = "nightly")]
pub mod typed_arena;

pub mod dropless_arena;
