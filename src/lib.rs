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
//! use lookups::{LkupHashMap, IndexLookup, Lookup};
//!
//! #[derive(PartialEq, Debug)]
//! struct Car {
//!     id: usize,
//!     name: String,
//! }
//!
//! // create a new Lookup HashMap with a Index-lookup
//! // that only supported unique `Key`s: `Car::id` (usize)
//! let mut map = LkupHashMap::new(IndexLookup::with_unique_key() ,|c: &Car| c.id);
//!
//! map.insert(String::from("BMW"),  Car{id: 0, name: "BMW".into()});
//! map.insert(String::from("Audi"), Car{id: 5, name: "Au".into()});
//! map.insert(String::from("VW"),   Car{id: 2, name: "VW".into()});
//!
//! // conventionally HashMap access with Key (name: String)
//! assert!(map.contains_key("VW"));
//! // lookup with Key (id: usize)
//! assert!(map.contains_lkup_key(2));
//!
//! // update one entry by lookup-key
//! assert_eq!(1, map.update_by_key(5, |c| c.name = "Audi".into()));
//!
//! // get a Car by an given: id
//! assert_eq!(Some(&Car{id: 5, name: "Audi".into()}), map.get_by_lkup_key(5).next());
//!
//!
//! // create a View: a subset from the Lookups (defined by the given Keys)
//! let view = map.create_lkup_view([0, 2]);
//!
//! assert_eq!(Some(&Car{id: 0, name: "BMW".into()}), view.get_by_key(0).next());
//! assert_eq!(vec![&Car{id: 0, name: "BMW".into()}, &Car{id: 2, name: "VW".into()}],
//!            view.items().collect::<Vec<_>>());
//!
//! // get min and max key
//! assert_eq!(Some(0), view.min_key());
//! assert_eq!(Some(2), view.max_key());
//!```
pub mod collections;
pub mod lookup;

pub use collections::list::rw::LkupVec;
pub use collections::map::rw::LkupHashMap;

pub use lookup::hash::HashLookup;
pub use lookup::index::IndexLookup;

pub use lookup::store::Lookup;
