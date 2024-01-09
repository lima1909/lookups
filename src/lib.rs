//! __lookups__ is a crate for extending already existing collection (Vec, Slice, Map, ...)
//! with additional lookup functionalities, so you have a faster access to your data as with an `iterator` or a `search algorithm`.
//! This wrapper is just as easy to use as the given (original) collection.
//!
//! The fast access can be achieved by using different methods, like;
//!
//! - hashing
//! - indexing
//! - search trees
//! - ...
//!
//! ## Disadvantage
//!
//! - it is more memory required. In addition to the user data, data for the _hash_, _index_ are also stored.
//! - the write operation are slower, because for every wirte operation is a another on (for storing the lookup data) necessary
//!
//! ## How can I use it?
//!
//!```ignore
//! ...
//!
//!```
pub mod collections;
pub mod lookup;
