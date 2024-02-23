//! The `collections` module contains the collections implementations which are using the lookups.
//!

pub mod list;
pub mod map;

use crate::lookup::store::Lookup;
use std::ops::Index;

/// A `Retriever` is the main interface for get Items by an given `Lookup`.
pub struct Retriever<L, I> {
    lookup: L,
    items: I,
}

impl<L, I> Retriever<L, I> {
    /// Create a new instance of an [`Retriever`].
    pub const fn new(lookup: L, items: I) -> Self {
        Self { lookup, items }
    }

    /// Checks whether the `Key` in the collection exists.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::lookup::MultiPosIndex;
    /// use lookups::collections::list::ro::LkupList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = [Car(5, "BMW".into()), Car(1, "Audi".into())];
    ///
    /// let v = LkupList::<MultiPosIndex, _>::new(|c| c.0, cars);
    ///
    /// assert!(v.lkup().contains_key(1));
    /// assert!(!v.lkup().contains_key(99));
    /// ```
    pub fn contains_key<Q>(&self, key: Q) -> bool
    where
        L: Lookup<Q>,
    {
        self.lookup.key_exist(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// # Example
    ///
    /// ```
    /// use lookups::lookup::MultiPosIndex;
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
    /// let v = LkupList::<MultiPosIndex, _>::new(Car::id, cars);
    ///
    /// assert_eq!(vec![&Car(1, "Audi".into())], v.lkup().get_by_key(1).collect::<Vec<_>>());
    /// ```
    pub fn get_by_key<'a, Q>(&'a self, key: Q) -> impl Iterator<Item = &'a I::Output>
    where
        I: Index<&'a L::Pos>,
        L: Lookup<Q>,
        Q: 'a,
    {
        self.lookup.pos_by_key(key).iter().map(|p| &self.items[p])
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
    /// use lookups::lookup::MultiPosIndex;
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
    /// let v = LkupList::<MultiPosIndex, _>::new(Car::id, cars);
    ///
    /// assert_eq!(
    ///     vec![&Car(5, "BMW".into()), &Car(1, "Audi".into())],
    ///     v.lkup().get_by_many_keys([5, 1]).collect::<Vec<_>>()
    /// );
    /// ```
    pub fn get_by_many_keys<'a, It, Q>(&'a self, keys: It) -> impl Iterator<Item = &'a I::Output>
    where
        It: IntoIterator<Item = Q> + 'a,
        I: Index<&'a L::Pos>,
        L: Lookup<Q>,
        Q: 'a,
    {
        self.lookup.pos_by_many_keys(keys).map(|p| &self.items[p])
    }
}

impl<L, I> std::ops::Deref for Retriever<L, I>
where
    L: std::ops::Deref,
{
    type Target = L::Target;

    fn deref(&self) -> &Self::Target {
        self.lookup.deref()
    }
}
