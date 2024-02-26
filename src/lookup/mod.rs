//! The `lookup` module contains the structure for storing and accessing the lookup implementations.
//!
//! A `Lookup` is a mapping from a `Key` to one ore more `Positions`:
//!
//! - the `Key`: ist the value, what to look for (like a key by a HashMap)
//! - the `Position`: is the location, where the `Key` is saved (the index, by Vec, or the key, by a HashMap)
//!   (`Position` is equivalent to the [`std::ops::Index`]). There are two kinds of `Positions`
//!     - Unique Position: there is exactly one `Position` (e.g: [`hash::UniquePosHash`], [`index::UniquePosIndex`])
//!     - Multi Position: there are many `Positions` possible (e.g: [`hash::MultiPosHash`], [`index::MultiPosIndex`])
//!
//! In the moment, there are two `Lookup` implementations
//! - hashing based lookup (the implementaion is a `HashMap`)  (e.g: [`hash::HashLookup`])
//! - index base lookup (the lookup carried out by the Index from a `Vec`) (e.g: [`index::IndexLookup`])
//!
pub mod hash;
pub mod index;
pub mod store;
