//! The `collections` module contains the collections implementations which are using the lookups.
//!

pub mod list;
pub mod map;

use crate::lookup::store::{self, Positions, Retriever};
use std::ops::Index;

pub use crate::collections::list::rw::LkupVec;
pub use crate::collections::map::rw::LkupHashMap;

/// A `View` is a sub set from a `Lookup`.
pub struct View<R, I> {
    view: store::View<R>,
    items: I,
}

impl<R, I> View<R, I> {
    /// Create a new instance of an [`View`]r.
    pub const fn new(view: store::View<R>, items: I) -> Self {
        Self { view, items }
    }

    /// Checks whether the `Key` in the collection exists.
    ///
    /// # Example
    ///
    /// ```ignore
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
        self.view.key_exist(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// # Example
    ///
    /// ```ignore
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
        self.view.pos_by_key(key).iter().map(|p| &self.items[p])
    }

    /// Combines all given `keys` with an logical `OR`.
    ///
    /// # Example:
    ///
    /// ```ignore
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
        self.view.pos_by_many_keys(keys).map(|p| &self.items[p])
    }

    /// Return all items for the given `View`.
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
        self.view.positions().map(|p| &self.items[p])
    }
}

impl<L, I> std::ops::Deref for View<L, I>
where
    L: std::ops::Deref,
{
    type Target = L::Target;

    fn deref(&self) -> &Self::Target {
        self.view.deref()
    }
}
