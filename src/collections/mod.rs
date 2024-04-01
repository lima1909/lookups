//! The `collections` module contains the collections implementations which are using the lookups.
//!

pub mod list;
pub mod map;

use crate::lookup::store::{Positions, Retriever};
use std::ops::Index;

pub use crate::collections::list::rw::LkupVec;
pub use crate::collections::map::rw::LkupHashMap;

/// A `Retrieve`r is the main interface for get Items by an given `Lookup`.
pub struct Retrieve<R, I> {
    retriever: R,
    items: I,
}

impl<R, I> Retrieve<R, I> {
    /// Create a new instance of an [`Retrieve`]r.
    pub const fn new(retriever: R, items: I) -> Self {
        Self { retriever, items }
    }

    /// Checks whether the `Key` in the collection exists.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::{collections::list::ro::LkupList, IndexLookup, Lookup};
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = [Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LkupList::new(IndexLookup::with_multi_keys(), |c| c.0, cars);
    ///
    /// assert!(v.lkup().contains_key(1));
    /// assert!(!v.lkup().contains_key(99));
    /// ```
    pub fn contains_key<Q>(&self, key: Q) -> bool
    where
        R: Retriever<Q>,
    {
        self.retriever.key_exist(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::{collections::list::ro::LkupList, IndexLookup, Lookup};
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = [Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LkupList::new(IndexLookup::with_multi_keys(), Car::id, cars);
    ///
    /// assert_eq!(vec![&Car(1, "Audi".into())], v.lkup().get_by_key(1).collect::<Vec<_>>());
    /// ```
    pub fn get_by_key<'a, Q>(&'a self, key: Q) -> impl Iterator<Item = &'a I::Output>
    where
        I: Index<&'a R::Pos>,
        R: Retriever<Q>,
        Q: 'a,
    {
        self.retriever
            .pos_by_key(key)
            .iter()
            .map(|p| &self.items[p])
    }

    /// Combines all given `keys` with an logical `OR`.
    ///
    /// # Example:
    ///
    /// ```
    /// use lookups::{collections::list::ro::LkupList, IndexLookup, Lookup};
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = [Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LkupList::new(IndexLookup::with_multi_keys(), Car::id, cars);
    ///
    /// assert_eq!(
    ///     vec![&Car(5, "BMW".into()), &Car(1, "Audi".into())],
    ///     v.lkup().get_by_many_keys([5, 1]).collect::<Vec<_>>()
    /// );
    /// ```
    pub fn get_by_many_keys<'a, It, Q>(&'a self, keys: It) -> impl Iterator<Item = &'a I::Output>
    where
        It: IntoIterator<Item = Q> + 'a,
        I: Index<&'a R::Pos>,
        R: Retriever<Q>,
        Q: 'a,
    {
        self.retriever
            .pos_by_many_keys(keys)
            .map(|p| &self.items[p])
    }

    /// Return all items for the given `Retriever`.
    ///
    /// # Example:
    ///
    /// ```
    /// use lookups::{collections::list::ro::LkupList, IndexLookup, Lookup};
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = [Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LkupList::new(IndexLookup::with_multi_keys(), Car::id, cars);
    /// let view = v.create_lkup_view([5]);
    ///
    /// assert_eq!(vec![&Car(5, "BMW".into())], view.items().collect::<Vec<_>>());
    /// ```
    pub fn items<'a>(&'a self) -> impl Iterator<Item = &'a I::Output>
    where
        I: Index<&'a R::Pos>,
        R: Positions<'a>,
    {
        self.retriever.positions().map(|p| &self.items[p])
    }
}

impl<L, I> std::ops::Deref for Retrieve<L, I>
where
    L: std::ops::Deref,
{
    type Target = L::Target;

    fn deref(&self) -> &Self::Target {
        self.retriever.deref()
    }
}
