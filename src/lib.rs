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
//! use lookups::{collections::LkupHashMap, lookup::UniquePosIndex};
//!
//! // create a new Lookup HashMap: LkupMap with a UniquePosIndex
//! let mut map = LkupHashMap::<UniquePosIndex<usize, String>, _, _, _>::new(|c: &Car| c.id);
//! map.insert(String::from("BMW"),  Car{id: 0, name: "BMW".into()});
//! map.insert(String::from("Audi"), Car{id: 5, name: "Audi".into()});
//! map.insert(String::from("VW"),   Car{id: 2, name: "VW".into()});
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
