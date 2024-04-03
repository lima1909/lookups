//! `Read write` implementations for lookup collections `Vec`.
//!

use crate::{
    collections::list::ro,
    lookup::store::{position::KeyPosition, Lookup, Retriever, Store},
};
use std::{fmt::Debug, ops::Deref};

/// [`LkupVec`] is a [`std::vec::Vec`] with one `Lookup`.
///
/// # Example
///
/// ```
/// #[derive(PartialEq, Debug)]
/// struct Person {
///     id: usize,
///     name: String,
/// }
///
/// use lookups::{LkupVec, HashLookup, Lookup};
///
/// let mut vec = LkupVec::new(HashLookup::with_multi_keys(), |p: &Person| p.name.clone());
///
/// vec.push(Person{id: 0, name: "Paul".into()});
/// vec.push(Person{id: 5, name: "Mario".into()});
/// vec.push(Person{id: 2, name: "Jasmin".into()});
///
/// assert!(vec.contains_lkup_key("Paul")); // lookup with a given Key
///
/// assert_eq!(
///     &Person{id: 5, name:  "Mario".into()},
///     // get a Person by an given Key: "Mario"
///     vec.get_by_lkup_key("Mario").next().unwrap()
/// );
///
/// assert_eq!(
///     vec![&Person{id: 0, name:  "Paul".into()}, &Person{id: 2, name:  "Jasmin".into()}],
///     // get many a Person by an many given Key
///     vec.get_by_many_lkup_keys(["Paul", "Jasmin"]).collect::<Vec<_>>(),
/// );
/// ```
///

#[derive(Debug, Clone)]
pub struct LkupVec<S, F, I> {
    field: F,
    inner: ro::LkupList<S, Vec<I>>,
}

impl<S, F, I> LkupVec<S, F, I>
where
    S: Store<Pos = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new<L, P>(lookup: L, field: F) -> Self
    where
        L: Lookup<S, P>,
        P: KeyPosition<Pos = usize>,
    {
        Self {
            inner: ro::LkupList::new(lookup, &field, Vec::new()),
            field,
        }
    }
}

impl<S, F, I> Deref for LkupVec<S, F, I> {
    type Target = ro::LkupList<S, Vec<I>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, F, I> LkupVec<S, F, I>
where
    S: Store<Pos = usize>,
    F: Fn(&I) -> S::Key,
{
    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) -> usize {
        let idx = self.inner.items.len();
        self.inner.store.insert((self.field)(&item), idx);
        self.inner.items.push(item);
        idx
    }

    /// Update an existing `Item` on given index from the List.
    /// If the index exist, the method returns an `Some` with reference to the updated Item.
    /// If not, the method returns `None`.
    pub fn update<U>(&mut self, index: usize, mut update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.inner.items.get_mut(index).map(|item| {
            let old_key = (self.field)(item);
            update(item);
            let new_key = (self.field)(item);

            self.inner.store.update(old_key, index, new_key);
            &*item
        })
    }

    /// The Item on index in the list will be removed.
    ///
    /// ## Hint:
    /// The remove is a swap_remove ([`std::vec::Vec::swap_remove`]).
    pub fn remove(&mut self, index: usize) -> Option<I> {
        if self.inner.items.is_empty() {
            return None;
        }

        let last_idx = self.inner.items.len() - 1;
        // index out of bound
        if index > last_idx {
            return None;
        }

        // last item in the list
        if index == last_idx {
            let rm_item = self.inner.items.remove(index);
            self.inner.store.delete((self.field)(&rm_item), &index);
            return Some(rm_item);
        }

        // remove item and entry in store and swap with last item
        let rm_item = self.inner.items.swap_remove(index);
        self.inner.store.delete((self.field)(&rm_item), &index);

        // formerly last item, now item on index, the swap for the store
        let curr_item = &self.inner.items[index];
        self.inner.store.delete((self.field)(curr_item), &last_idx);
        self.inner.store.insert((self.field)(curr_item), index);

        Some(rm_item)
    }

    /// Call `update`-function of all items by a given `Key`.
    /// Return value is the size of updated Items.
    pub fn update_by_key<Q, U>(&mut self, key: Q, mut update: U) -> usize
    where
        S: Retriever<Q, Pos = usize>,
        U: FnMut(&mut I),
    {
        let mut update_count = 0;

        #[allow(clippy::unnecessary_to_owned)]
        for idx in self.store.pos_by_key(key).to_vec() {
            if self.update(idx, &mut update).is_some() {
                update_count += 1;
            }
        }

        update_count
    }

    /// Remove all items by a given `Key`.
    /// Return value is the size of removed Items.
    pub fn remove_by_key<Q>(&mut self, key: Q) -> usize
    where
        S: Retriever<Q, Pos = usize>,
        Q: Clone,
    {
        let mut remove_count = 0;

        while let Some(idx) = self.store.pos_by_key(key.clone()).iter().next() {
            if self.remove(*idx).is_some() {
                remove_count += 1;
            }
        }

        remove_count
    }
}

//
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::hash::HashLookup;

    #[derive(PartialEq, Debug, Clone)]
    struct Person {
        id: usize,
        name: String,
    }

    impl Person {
        fn new(id: usize, name: &str) -> Self {
            Self {
                id,
                name: name.into(),
            }
        }

        fn id(&self) -> usize {
            self.id
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[test]
    fn lkupvec() {
        let mut v = LkupVec::new(HashLookup::with_multi_keys(), Person::name);
        v.push(Person::new(1, "Anna"));
        v.push(Person::new(0, "Paul"));

        assert!(!v.is_empty());
        assert_eq!(2, v.len());

        assert_eq!(&Person::new(1, "Anna"), &v[0]);

        assert_eq!(
            &Person::new(1, "Anna"),
            v.get_by_lkup_key("Anna").next().unwrap()
        );

        // id 101 not exist
        // assert_eq!(None, v.update(101, |p| { p.id = 102 }));
        assert_eq!(
            Some(&Person::new(99, "Anna")),
            v.update(0, |p| { p.id = 99 })
        );
        assert_eq!(
            &Person::new(99, "Anna"),
            v.get_by_lkup_key("Anna").next().unwrap()
        );

        let view = v.create_lkup_view([String::from("Paul")]);
        assert_eq!(
            &Person::new(0, "Paul"),
            view.get_by_key("Paul").next().unwrap()
        );
        assert!(view.get_by_key("Anna").next().is_none());
    }

    #[test]
    fn remove() {
        let mut v = LkupVec::new(HashLookup::with_multi_keys(), Person::id);
        v.push(Person::new(1, "Anna"));
        v.push(Person::new(2, "Paul"));

        assert_eq!(2, v.len());

        assert_eq!(Some(Person::new(1, "Anna")), v.remove(0));
        assert_eq!(1, v.len());
        assert_eq!(Person::new(2, "Paul"), v[0]);

        assert_eq!(Some(Person::new(2, "Paul")), v.remove(0));
        assert_eq!(0, v.len());

        assert_eq!(None, v.remove(0));
        assert_eq!(0, v.len());
    }

    #[test]
    fn remove_by_key() {
        let mut v = LkupVec::new(HashLookup::with_multi_keys(), Person::id);
        v.push(Person::new(1, "Anna"));
        v.push(Person::new(2, "Paul"));

        assert_eq!(2, v.len());

        // key not exist
        v.remove_by_key(&99);
        assert_eq!(2, v.len());

        // remove key = 1 (Anna)
        v.remove_by_key(&1);
        assert_eq!(1, v.len());

        assert!(!v.contains_lkup_key(&1));
        assert!(v.contains_lkup_key(&2));

        v.remove_by_key(&2);
        assert_eq!(0, v.len());
    }
}
