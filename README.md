# Lookups [![Build Status]][Build Action] [![Coverage Status]][Coverage Action][![Latest Version]][crates.io]  

[Build Status]: https://github.com/lima1909/lookups/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/lookups/actions
[Coverage Status]: https://codecov.io/gh/lima1909/lookups/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/lookups
[Latest Version]: https://img.shields.io/crates/v/lookups.svg
[crates.io]: https://crates.io/crates/lookups


Improve the data retrieval operations for collections.

# Overview

__lookups__ is a crate for extending already existing collections (`std::vec::Vec`, `std::collections::HashMap`, ...)
with additional lookup functionalities, so you have a faster access to your data as with an `iterator` or a `search algorithm`.
This wrapper is just as easy to use as the given (original) collection.

The fast access can be achieved by using different methods, like;

- hash tables
- indexing
- ...

### Disadvantage

- it is more memory required. In addition to the user data, data for the _hash_, _index_ are also stored.
- the write operation are slower, because for every wirte operation is an another one (for storing the lookup data) necessary

### How can I use it?

```rust
#[derive(PartialEq, Debug)]
struct Car {
    id: usize,
    name: String,
}

let cars = [
    (String::from("BMW"),  Car{id: 0, name: "BMW".into()}),
    (String::from("Audi"), Car{id: 5, name: "Audi".into()}),
    (String::from("VW"),   Car{id: 2, name: "VW".into()}),
];

use lookups::{collections::ro::LHashMap, lookup::UniquePosIndex};

// create a new Lookup HashMap: LHashMap with a UniquePosIndex
let map = LHashMap::<UniquePosIndex<_>, _, _>::new(|c| c.id, cars);

// conventionally HashMap access with Key (name: String)
assert!(map.contains_key("VW"));
// lookup with Key (id: usize)
assert!(map.lkup().contains_key(2));

// get a Car by an given Key
assert_eq!(map.lkup().get_by_key(5).next(), Some(&Car{id: 5, name: "Audi".into()}));

// create a View
let view = map.create_view([0, 2]);

assert_eq!(view.get_by_key(0).next(), Some(&Car{id: 0, name: "BMW".into()}));
```
