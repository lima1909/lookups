//! The `collections` module contains the collections implementations which are using the lookups.
//!

pub mod list;
pub mod map;

use crate::lookup::store::Retriever;
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
    /// use lookups::{IndexLookup, Lookup};
    /// use lookups::collections::list::ro::LkupList;
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
    /// use lookups::{IndexLookup, Lookup};
    /// use lookups::collections::list::ro::LkupList;
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
    ///```text
    /// get_by_many_keys([2, 5, 6]) => get_by_key(2) OR get_by_key(5) OR get_by_key(6)
    /// get_by_many_keys(2..6]) => get_by_key(2) OR get_by_key(3) OR get_by_key(4) OR get_by_key(5)
    /// ```
    ///
    /// # Example:
    ///
    /// ```
    /// use lookups::{IndexLookup, Lookup};
    /// use lookups::collections::list::ro::LkupList;
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

/// `Edit` describe the operations for changing (update and remove) `Items` in a collection.
pub trait Edit<Idx, Item> {
    type Retriever;

    /// Update the item on the given collection index.
    fn update<U>(&mut self, index: Idx, update: U) -> Option<&Item>
    where
        U: FnMut(&mut Item);

    /// The Item on the given position will be removed from the collection.
    fn remove(&mut self, index: Idx) -> Option<Item>;

    // Get all `Indices` by a given `Key`
    fn get_indices_by_key<Q>(&self, key: Q) -> &[Idx]
    where
        Self::Retriever: Retriever<Q, Pos = Idx>;

    /// Call `update`-function of all items by a given `Key`.
    fn update_by_key<Q, U>(&mut self, key: Q, mut update: U) -> usize
    where
        Self::Retriever: Retriever<Q, Pos = Idx>,
        U: FnMut(&mut Item),
        Idx: Clone,
    {
        let mut update_count = 0;

        #[allow(clippy::unnecessary_to_owned)]
        for idx in self.get_indices_by_key(key).to_vec() {
            if self.update(idx, &mut update).is_some() {
                update_count += 1;
            }
        }

        update_count
    }

    /// Remove all items by a given `Key`.
    fn remove_by_key<Q>(&mut self, key: Q) -> usize
    where
        Self::Retriever: Retriever<Q, Pos = Idx>,
        Idx: Clone,
        Q: Clone,
    {
        let mut remove_count = 0;

        while let Some(idx) = self.get_indices_by_key(key.clone()).iter().next() {
            if self.remove(idx.clone()).is_some() {
                remove_count += 1;
            }
        }

        remove_count
    }
}
