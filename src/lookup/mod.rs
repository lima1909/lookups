//! The `lookup` module contains the structure for storing and accessing the lookup implementations.
//!
//! A `Lookup` is a mapping from a `Key` to one ore more `Positions`:
//!
//! - the `Key`: ist the value, what to look for (like a key by a HashMap)
//! - the `Position`: is the location, where the `Key` is saved (the index, by Vec, or the key, by a HashMap)
//!   (`Position` is equivalent to the [`std::ops::Index`]).
//!   There are two kinds of `Key`s
//!     - Unique: there is exactly one `Key`
//!     - Multi : there are many `Key`s possible
//!
//! In the moment, there are two `Lookup` implementations
//! - hashing based lookup (the implementaion is a `HashMap`)  (e.g: [`hash::HashStore`])
//! - index base lookup (the lookup carried out by the Index from a `Vec`) (e.g: [`index::IndexStore`])
//!
pub mod hash;
pub mod index;
pub mod store;
