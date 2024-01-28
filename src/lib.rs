//! __lookups__ is a crate for extending already existing collections (`std::vec::Vec`, `std::collections::HashMap`, ...)
//! with additional lookup functionalities, so you have a faster access to your data as with an `iterator` or a `search algorithm`.
//! This wrapper is just as easy to use as the given (original) collection.
//!
//! The fast access can be achieved by using different methods, like;
//!
//! - hash tables
//! - indexing
//! - ...
//!
//! ## Disadvantage
//!
//! - it is more memory required. In addition to the user data, data for the _hash_, _index_ are also stored.
//! - the write operation are slower, because for every wirte operation is an another one (for storing the lookup data) necessary
//!
//! ## How can I use it?
//!
//!```
//! use lookups::{collections::ro::LHashMap, lookup::UniqueUIntLookup};
//!
//! #[derive(PartialEq, Debug)]
//! struct Car {
//!     id: usize,
//!     name: String,
//! }
//!
//! let cars = [
//!     (String::from("BMW"),  Car{id: 0, name: "BMW".into()}),
//!     (String::from("Audi"), Car{id: 5, name: "Audi".into()}),
//!     (String::from("VW"),   Car{id: 2, name: "VW".into()}),
//! ];
//!
//! // create a new Lookup HashMap: LHashMap with a UniqueUIntLookup
//! let map = LHashMap::<UniqueUIntLookup<_, _>, _, _>::new(|c| c.id, cars);
//!
//! assert!(map.contains_key("VW"));       // conventionally HashMap access with Key (String)
//! assert!(map.lkup().contains_key(2));   // lookup with Key (id: usize)
//!
//! assert_eq!(
//!     &Car{id: 5, name: "Audi".into()},
//!     // get a Car by an given Key
//!     map.lkup().get_by_key(5).next().unwrap()
//! );
//!
//! assert_eq!(
//!     vec![&Car{id: 0, name: "BMW".into()}, &Car{id: 2, name: "VW".into()}],
//!     // get many Cars by given many Keys
//!     map.lkup().get_by_many_keys([0, 2]).collect::<Vec<_>>(),
//! );
//!```
pub mod collections;
pub mod lookup;

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;
