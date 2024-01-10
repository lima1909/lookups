//! The `collections` module contains the collections implementations which are using the lookups.
//!

use crate::lookup::store::{item::ItemAt, Lookup};
use std::marker::PhantomData;

pub mod ro;

/// A `Retriever` is the main interface for get Items by an given `Lookup`.
pub struct Retriever<'a, L, I, Q> {
    lookup: &'a L,
    items: I,
    _q: PhantomData<Q>,
}

impl<'a, L, I, Q> Retriever<'a, L, I, Q>
where
    L: Lookup<Q>,
{
    /// Create a new instance of an [`Retriever`].
    pub const fn new(lookup: &'a L, items: I) -> Self {
        Self {
            lookup,
            items,
            _q: PhantomData,
        }
    }

    /// Checks whether the `Key` in the collection exists.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::lookup::MultiUIntLookup;
    /// use lookups::collections::ro::LVec;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LVec::<MultiUIntLookup, _>::new(|c| c.0, cars);
    ///
    /// assert!(v.idx().contains_key(1));
    /// assert!(!v.idx().contains_key(99));
    /// ```
    pub fn contains_key(&self, key: Q) -> bool {
        self.lookup.key_exist(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::lookup::MultiUIntLookup;
    /// use lookups::collections::ro::LVec;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = vec![Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LVec::<MultiUIntLookup, _>::new(Car::id, cars);
    ///
    /// assert_eq!(vec![&Car(1, "Audi".into())], v.idx().get_by_key(1).collect::<Vec<_>>());
    /// ```
    pub fn get_by_key(&self, key: Q) -> impl Iterator<Item = &I::Output>
    where
        I: ItemAt<L::Pos>,
    {
        self.lookup
            .pos_by_key(key)
            .iter()
            .map(|pos| self.items.item(pos))
    }
}
