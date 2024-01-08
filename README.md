# Lookups [![Latest Version]][crates.io]  

[Latest Version]: https://img.shields.io/crates/v/lookups.svg
[crates.io]: https://crates.io/crates/lookups


Coming soon ...

Improve the data retrieval operations for collections.

# Overview

__lookups__ is a crate for extending already existing collection (Vec, Slice, Map, ...) by additional and performant lookup functionalities.

This means faster than an `iterator` or a `search algorithm`.

This crate provides wrapper for the Rust collections, which extends the given collection with fast retrieval operations.
This wrapper is just as easy to use as the given (original) collection.

The fast access can be achieved by using different methods, like;

- hashing
- indexing
- search trees
- ...

### Disadvantage

It is more memory required. In addition to the user data, data for the _hash_, _index_ are also stored.
