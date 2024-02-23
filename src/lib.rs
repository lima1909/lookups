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
//! #[derive(PartialEq, Debug)]
//! struct Car {
//!     id: usize,
//!     name: String,
//! }
//!
//! let mut cars = std::collections::HashMap::new();
//! cars.insert(String::from("BMW"),  Car{id: 0, name: "BMW".into()});
//! cars.insert(String::from("Audi"), Car{id: 5, name: "Audi".into()});
//! cars.insert(String::from("VW"),   Car{id: 2, name: "VW".into()});
//!
//! use lookups::{collections::map::ro::LHashMap, lookup::UniquePosIndex};
//!
//! // create a new Lookup HashMap: LHashMap with a UniquePosIndex
//! let map = LHashMap::<UniquePosIndex<_, _>, _>::new(|c| c.id, cars);
//!
//! // conventionally HashMap access with Key (name: String)
//! assert!(map.contains_key("VW"));
//! // lookup with Key (id: usize)
//! assert!(map.lkup().contains_key(2));
//!
//! // get a Car by an given Key
//! assert_eq!(map.lkup().get_by_key(5).next(), Some(&Car{id: 5, name: "Audi".into()}));
//! assert_eq!((map.lkup().min_key(), map.lkup().max_key()), (Some(0), Some(5)));
//!
//! // create a View: a subset from the Lookups (defined by the given Keys)
//! let view = map.create_lkup_view([0, 2]);
//!
//! assert_eq!(view.get_by_key(0).next(), Some(&Car{id: 0, name: "BMW".into()}));
//! assert_eq!((view.min_key(), view.max_key()), (Some(0), Some(2)));
//!```
pub mod collections;
pub mod lookup;

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;
