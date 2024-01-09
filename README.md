# Lookups [![Build Status]][Build Action] [![Coverage Status]][Coverage Action][![Latest Version]][crates.io]  

[Build Status]: https://github.com/lima1909/lookups/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/lookups/actions
[Coverage Status]: https://codecov.io/gh/lima1909/lookups/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/lookups
[Latest Version]: https://img.shields.io/crates/v/lookups.svg
[crates.io]: https://crates.io/crates/lookups


__Coming soon ...__

Improve the data retrieval operations for collections.

# Overview

__lookups__ is a crate for extending already existing collection (Vec, Slice, Map, ...)
with additional lookup functionalities, so you have a faster access to your data as with an `iterator` or a `search algorithm`.
This wrapper is just as easy to use as the given (original) collection.

The fast access can be achieved by using different methods, like;

- hashing
- indexing
- search trees
- ...

### Disadvantage

- it is more memory required. In addition to the user data, data for the _hash_, _index_ are also stored.
- the write operation are slower, because for every wirte operation is a another on (for storing the lookup data) necessary

### How can I use it?

```rust
...

```
