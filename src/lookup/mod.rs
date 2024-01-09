//! The `lokkup` module contains the structure for storing and accessing the lookup implementations.
//!
pub mod map;
pub mod store;

pub use map::{MultiMapLookup, UniqueMapLookup};
