//! The module contains lookups which are using the _index_ position (the `Key`) in a `Vec` find all `Position`s.
//!
//! This lookup is well suited for __consecutive numbers__,
//! which starts by `0` or `1`, and do __not__ have any great __gaps__ in beetween.
//! The `Key` musst implement the trait: `Into<usize>`.
//!
//! Gaps are a disadvantage by:
//! - inserting new `Key`s (the Vec mus grow strongly: [`IndexLookup::insert`])
//! - and by getting all `Key`s with [`IndexLookupExt::keys`]
//!
//! One use case for this conditions are __primary keys__ (e.g. artificially created).
//!
//! ## Advantages:
//!
//! - the finding of an `Key` is very fast (you can __directly__ jump to the `Key`)
//!
use crate::lookup::store::{
    KeyPosition, Lookup, MultiKeyPositon, Store, UniqueKeyPositon, View, ViewCreator,
};
use std::{marker::PhantomData, ops::Deref};

/// Implementation for a `Index` with unique `Position`.
pub type UniquePosIndex<X = usize> = IndexLookup<UniqueKeyPositon<X>, X>;
/// Implementation for a `Index` with multi `Position`s.
pub type MultiPosIndex<X = usize> = IndexLookup<MultiKeyPositon<X>, X>;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
pub struct IndexLookup<P, X = usize> {
    inner: Vec<Option<(usize, P)>>,
    min_index: usize,
    max_index: usize,
    _x: PhantomData<X>,
}

impl<P, X> Lookup<usize> for IndexLookup<P, X>
where
    P: KeyPosition<X>,
{
    type Pos = X;

    fn key_exist(&self, key: usize) -> bool {
        matches!(self.inner.get(key), Some(Some(_)))
    }
    fn pos_by_key(&self, key: usize) -> &[Self::Pos] {
        match self.inner.get(key) {
            Some(Some((_, p))) => p.as_slice(),
            _ => &[],
        }
    }
}

impl<P, X> Store for IndexLookup<P, X>
where
    P: KeyPosition<X> + Clone,
{
    type Key = usize;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        // if necessary (len <= idx), than double the vec
        if self.inner.len() <= key {
            const PRE_ALLOC_SIZE: usize = 100;
            self.inner.resize(key + PRE_ALLOC_SIZE, None);
        }

        // insert new key and pos
        match self.inner.get_mut(key) {
            Some(Some((_, p))) => p.add_pos(pos),
            _ => self.inner[key] = Some((key, P::new(pos))),
        }

        // define new max index
        if self.max_index < key {
            self.max_index = key;
        }
        // define new min index
        if self.min_index > key {
            self.min_index = key;
        }
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        if let Some(Some((_, rm_idx))) = self.inner.get_mut(key) {
            if rm_idx.remove_pos(pos) {
                // las pos was deleted
                self.inner[key] = None;

                // define new max index
                if self.max_index == key {
                    self.max_index = self.find_new_max_index()
                }
                if self.min_index == key {
                    self.min_index = self.find_new_min_index()
                }
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            min_index: usize::MAX,
            max_index: 0,
            _x: PhantomData,
        }
    }
}

//
// ----------- internal (private) helper implementation --------------------------
//
impl<P, X> IndexLookup<P, X> {
    #[inline(always)]
    fn find_new_max_index(&self) -> usize {
        self.inner
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, o)| {
                if o.is_some() {
                    return Some(idx);
                }
                None
            })
            .unwrap_or_default()
    }

    #[inline(always)]
    fn find_new_min_index(&self) -> usize {
        self.inner
            .iter()
            .enumerate()
            .find_map(|(idx, o)| {
                if o.is_some() {
                    return Some(idx);
                }
                None
            })
            .unwrap_or(usize::MAX)
    }

    #[inline(always)]
    pub fn get_key_by_index(&self, index: usize) -> Option<usize> {
        self.inner.get(index)?.as_ref().map(|(k, _)| *k)
    }
}

/// A proxy for exposing [`IndexLookup`] specific extensions.
#[repr(transparent)]
pub struct IndexLookupExt<P, X>(IndexLookup<P, X>);

impl<P, X> Deref for IndexLookup<P, X> {
    type Target = IndexLookupExt<P, X>;

    fn deref(&self) -> &Self::Target {
        // SAFTY:
        // self is a valid pointer and
        // IndexLookupExt is repr(transparent) thus has the same memory layout like IndexLookup
        unsafe { &*(self as *const IndexLookup<P, X> as *const IndexLookupExt<P, X>) }
    }
}

impl<P, X> IndexLookupExt<P, X> {
    /// Returns all stored `Key`s.
    pub fn keys(&self) -> impl Iterator<Item = usize> + '_ {
        if self.0.min_index > self.0.max_index {
            &[]
        } else {
            &self.0.inner[self.0.min_index..=self.0.max_index]
        }
        .iter()
        .filter_map(|o| o.as_ref().map(|(key, _)| *key))
    }

    /// Returns smallest stored `Key`.
    pub fn min_key(&self) -> Option<usize> {
        self.0.get_key_by_index(self.0.min_index)
    }

    /// Returns greatest stored `Key`.
    pub fn max_key(&self) -> Option<usize> {
        self.0.get_key_by_index(self.0.max_index)
    }
}

impl<P, X> ViewCreator<'_, usize> for IndexLookup<P, X>
where
    P: KeyPosition<X> + Clone,
    X: Clone,
{
    type Key = usize;
    type Lookup = IndexLookup<P, X>;

    fn create_view<It>(&self, keys: It) -> View<Self::Lookup, usize>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut lkup = IndexLookup::<P, X>::with_capacity(self.inner.len());

        for key in keys {
            for p in self.pos_by_key(key) {
                lkup.insert(key, p.clone());
            }
        }

        View::new(lkup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gender() {
        #[derive(Debug, Clone, PartialEq)]
        #[repr(usize)]
        enum Gender {
            Female,
            Male,
            None,
        }

        impl PartialEq<usize> for Gender {
            fn eq(&self, other: &usize) -> bool {
                match self {
                    Gender::Female => 0 == *other,
                    Gender::Male => 1 == *other,
                    Gender::None => 2 == *other,
                }
            }
        }

        use Gender::*;

        let mut idx = MultiPosIndex::<_>::with_capacity(10);

        idx.insert(Female as usize, 10);
        idx.insert(Female as usize, 2);
        idx.insert(Male as usize, 1);
        idx.insert(None as usize, 0);

        assert_eq!(Female, idx.min_key().unwrap());
        assert_eq!(None, idx.max_key().unwrap());

        assert_eq!(vec![Female, Male, None], idx.keys().collect::<Vec<_>>());

        assert_eq!(&[2, 10], idx.pos_by_key(Female as usize));
    }

    #[test]
    fn create_view() {
        let mut idx = MultiPosIndex::<String>::with_capacity(0);
        idx.insert(0, String::from("a"));
        idx.insert(1, String::from("b"));
        idx.insert(2, String::from("c"));
        idx.insert(4, String::from("s"));

        assert!(idx.key_exist(0));

        let view = idx.create_view([1, 4]);
        assert!(!view.key_exist(0));
        assert!(!view.key_exist(2));

        assert!(view.key_exist(1));
        assert!(view.key_exist(4));

        assert_eq!(&[String::from("s")], view.pos_by_key(4));
        assert_eq!(&[String::from("b")], view.pos_by_key(1));

        assert_eq!(
            vec![&String::from("b"), &String::from("s")],
            view.pos_by_many_keys([0, 1, 2, 99, 4,]).collect::<Vec<_>>()
        );
    }

    mod min_max_keys {
        use super::*;

        #[test]
        fn by_insert() {
            let mut idx = MultiPosIndex::<usize>::with_capacity(0);

            // both not set
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
            assert_eq!(usize::MAX, idx.min_index);
            assert_eq!(0, idx.max_index);
            assert!(idx.keys().next().is_none());

            // first insert, max and min are equal
            idx.insert(1, 0);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(1), idx.max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(1, idx.max_index);
            assert_eq!(vec![1], idx.keys().collect::<Vec<_>>());

            // first insert, max and min are equal
            idx.insert(10, 1);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(10), idx.max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(10, idx.max_index);
            assert_eq!(vec![1, 10], idx.keys().collect::<Vec<_>>());

            idx.insert(11, 2);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(11), idx.max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(11, idx.max_index);
            assert_eq!(vec![1, 10, 11], idx.keys().collect::<Vec<_>>());

            idx.delete(10, &1);
            idx.delete(11, &2);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(1), idx.max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(1, idx.max_index);
            assert_eq!(vec![1], idx.keys().collect::<Vec<_>>());

            // remove last
            idx.delete(1, &0);
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
            assert_eq!(usize::MAX, idx.min_index);
            assert_eq!(0, idx.max_index);
            assert!(idx.keys().next().is_none());
        }

        #[test]
        fn by_insertwith_capacity() {
            let mut idx = MultiPosIndex::<usize>::with_capacity(5);
            // both not set
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
            assert!(idx.keys().next().is_none());

            // first insert, max and min are equal
            idx.insert(1, 1);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(1), idx.max_key());
            assert_eq!(vec![1], idx.keys().collect::<Vec<_>>());

            // min and max are different
            idx.insert(4, 4);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(4), idx.max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(4, idx.max_index);
            assert_eq!(vec![1, 4], idx.keys().collect::<Vec<_>>());

            // new min
            idx.insert(0, 0);
            assert_eq!(Some(0), idx.min_key());
            assert_eq!(Some(4), idx.max_key());
            assert_eq!(0, idx.min_index);
            assert_eq!(4, idx.max_index);
            assert_eq!(vec![0, 1, 4], idx.keys().collect::<Vec<_>>());

            // new max
            idx.insert(6, 6);
            assert_eq!(Some(0), idx.min_key());
            assert_eq!(Some(6), idx.max_key());
            assert_eq!(vec![0, 1, 4, 6], idx.keys().collect::<Vec<_>>());
        }

        #[test]
        fn by_delete() {
            let mut idx = MultiPosIndex::<usize>::with_capacity(5);

            // min/max not set
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());

            // remove not exist key/pos pair
            idx.delete(1, &1);
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());

            idx.insert(1, 1);
            idx.insert(2, 2);
            idx.insert(3, 2);
            idx.insert(4, 4);
            idx.insert(5, 5);
            assert_eq!(vec![1, 2, 3, 4, 5], idx.keys().collect::<Vec<_>>());

            // remove min key
            idx.delete(1, &1);
            assert_eq!(Some(2), idx.min_key());
            assert_eq!(Some(5), idx.max_key());
            assert_eq!(vec![2, 3, 4, 5], idx.keys().collect::<Vec<_>>());

            // remove no max and no min key
            idx.delete(4, &4);
            assert_eq!(Some(2), idx.min_key());
            assert_eq!(Some(5), idx.max_key());
            assert_eq!(vec![2, 3, 5], idx.keys().collect::<Vec<_>>());

            // remove min key
            idx.delete(2, &2);
            assert_eq!(Some(3), idx.min_key());
            assert_eq!(Some(5), idx.max_key());
            assert_eq!(vec![3, 5], idx.keys().collect::<Vec<_>>());

            // invalid pos, no key is removed
            idx.delete(3, &3);
            assert_eq!(Some(3), idx.min_key());
            assert_eq!(Some(5), idx.max_key());
            assert_eq!(vec![3, 5], idx.keys().collect::<Vec<_>>());

            // remove last key for pos 2
            idx.delete(3, &2);
            assert_eq!(Some(5), idx.min_key());
            assert_eq!(Some(5), idx.max_key());
            assert_eq!(vec![5], idx.keys().collect::<Vec<_>>());

            // remove last key
            idx.delete(5, &5);
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
            assert!(idx.keys().collect::<Vec<_>>().is_empty());
        }
    }

    #[test]
    fn store_and_lookup() {
        let mut idx = UniquePosIndex::<usize>::with_capacity(5);
        idx.insert(0, 0);
        idx.insert(1, 1);
        idx.insert(2, 2);
        idx.insert(4, 4);

        assert!(idx.key_exist(0));
        assert!(!idx.key_exist(1_000));

        assert_eq!(&[1], idx.pos_by_key(1));
        assert_eq!(&[2], idx.pos_by_key(2));
        assert_eq!(&[1usize; 0], idx.pos_by_key(1_000));

        // check many keys
        assert_eq!(
            vec![&0, &1, &4],
            idx.pos_by_many_keys([0, 1, 1_000, 4]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn store_and_lookup_complex_key() {
        #[derive(Debug, Clone, PartialEq)]
        struct Complex {
            id: usize,
            name: String,
        }

        let mut idx = UniquePosIndex::<Complex>::with_capacity(5);
        idx.insert(
            0,
            Complex {
                id: 1,
                name: String::from("0"),
            },
        );
        idx.insert(
            1,
            Complex {
                id: 2,
                name: String::from("1"),
            },
        );
        idx.insert(
            2,
            Complex {
                id: 3,
                name: String::from("2"),
            },
        );
        idx.insert(
            4,
            Complex {
                id: 4,
                name: String::from("4"),
            },
        );

        assert!(idx.key_exist(0));

        assert_eq!(
            &[Complex {
                id: 2,
                name: String::from("1"),
            }],
            idx.pos_by_key(1)
        );

        // check many keys
    }
}
