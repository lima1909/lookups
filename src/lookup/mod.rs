//! The `lokkup` module contains the structure for storing and accessing the lookup implementations.
//!
//! There are two kinds of `Positions`
//! - Unique, there is exactly one `Position` (e.g: [`hash::UniquePosHash`], [`index::UniquePosIndex`])
//! - Multi there are many `Positions` possible (e.g: [`hash::MultiPosHash`], [`index::MultiPosIndex`])
//!
//! For the `Key`s exist to lookup implementations
//! - hasing based lookup (the implementaion is a `HashMap`)  (e.g: [`hash::HashLookup`])
//! - index base lookup (the lookup carried out by the Index from a `Vec`) (e.g: [`index::IndexLookup`])
//!
pub mod hash;
pub mod index;
pub mod store;

pub use hash::{MultiPosHash, UniquePosHash};
pub use index::{MultiPosIndex, UniquePosIndex};
