//! The `lokkup` module contains the structure for storing and accessing the lookup implementations.
//!
pub mod map;
pub mod store;
pub mod uint;

pub use map::{MultiMapLookup, UniqueMapLookup};
pub use uint::{MultiUIntLookup, UniqueUIntLookup};
