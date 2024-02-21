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
    KeyPosition, KeyPositionAsSlice, Lookup, MultiKeyPositon, Store, UniqueKeyPositon, View,
    ViewCreator,
};
use std::ops::Deref;

/// Implementation for a `Index` with unique `Position`.
pub type UniquePosIndex<K = usize, X = usize> = IndexLookup<K, UniqueKeyPositon<X>>;
/// Implementation for a `Index` with multi `Position`s.
pub type MultiPosIndex<K = usize, X = usize> = IndexLookup<K, MultiKeyPositon<X>>;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
#[repr(transparent)]
pub struct IndexLookup<K, P>(Vec<Option<(K, P)>>);

impl<K, P> Lookup<K> for IndexLookup<K, P>
where
    K: Into<usize>,
    P: KeyPositionAsSlice,
{
    type Pos = P::Pos;

    fn key_exist(&self, key: K) -> bool {
        matches!(self.0.get(key.into()), Some(Some(_)))
    }
    fn pos_by_key(&self, key: K) -> &[Self::Pos] {
        match self.0.get(key.into()) {
            Some(Some((_, p))) => p.as_slice(),
            _ => &[],
        }
    }
}

impl<'a, K, P> ViewCreator<'a> for IndexLookup<K, P>
where
    K: Into<usize> + Clone,
    P: KeyPositionAsSlice + 'a,
{
    type Key = K;
    type Lookup = IndexLookup<K, &'a P>;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Lookup>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut lkup = Vec::new();
        lkup.resize(self.0.len(), None);

        for key in keys {
            let idx = key.into();
            if let Some(Some((k, p))) = self.0.get(idx) {
                lkup[idx] = Some((k.clone(), p));
            }
        }

        View::new(IndexLookup(lkup))
    }
}

impl<K, P> Store for IndexLookup<K, P>
where
    K: Into<usize> + Clone,
    P: KeyPosition + Clone,
{
    type Key = K;
    type Pos = P::Pos;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        let k = key.clone();
        let idx = k.into();

        // if necessary (len <= idx), than double the vec
        if self.0.len() <= idx {
            const PRE_ALLOC_SIZE: usize = 100;
            self.0.resize(idx + PRE_ALLOC_SIZE, None);
        }

        // insert new key and pos
        match self.0.get_mut(idx) {
            Some(Some((_, p))) => p.add_pos(pos),
            _ => self.0[idx] = Some((key, P::from_pos(pos))),
        }
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        let idx = key.into();

        if let Some(Some((_, rm_idx))) = self.0.get_mut(idx) {
            if rm_idx.remove_pos(pos) {
                // las pos was deleted
                self.0[idx] = None;
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

/// A proxy for exposing [`IndexLookup`] specific extensions.
#[repr(transparent)]
pub struct IndexLookupExt<K, P>(IndexLookup<K, P>);

impl<K, P> Deref for IndexLookup<K, P> {
    type Target = IndexLookupExt<K, P>;

    fn deref(&self) -> &Self::Target {
        // SAFTY:
        // self is a valid pointer and
        // IndexLookupExt is repr(transparent) thus has the same memory layout like IndexLookup
        unsafe { &*(self as *const IndexLookup<K, P> as *const IndexLookupExt<K, P>) }
    }
}

impl<K, P> IndexLookupExt<K, P>
where
    K: Clone,
{
    /// Returns all stored `Key`s.
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.0
             .0
            .iter()
            .filter_map(|o| o.as_ref().map(|(key, _)| key.clone()))
    }

    /// Returns smallest stored `Key`.
    pub fn min_key(&self) -> Option<K> {
        self.0 .0.iter().find_map(|o| {
            if let Some((key, _)) = o {
                return Some(key.clone());
            }
            None
        })
    }

    /// Returns greatest stored `Key`.
    pub fn max_key(&self) -> Option<K> {
        self.0 .0.iter().rev().find_map(|o| {
            if let Some((key, _)) = o {
                return Some(key.clone());
            }
            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gender() {
        #[derive(Debug, Clone, PartialEq)]
        enum Gender {
            Female,
            Male,
            None,
        }

        impl From<Gender> for usize {
            fn from(value: Gender) -> Self {
                match value {
                    Gender::Female => 0,
                    Gender::Male => 1,
                    Gender::None => 2,
                }
            }
        }

        use Gender::*;

        let mut idx = MultiPosIndex::with_capacity(10);

        idx.insert(Female, 10);
        idx.insert(Female, 2);
        idx.insert(Male, 1);
        idx.insert(None, 0);

        assert_eq!(Female, idx.min_key().unwrap());
        assert_eq!(None, idx.max_key().unwrap());

        assert_eq!(vec![Female, Male, None], idx.keys().collect::<Vec<_>>());

        assert_eq!(&[2, 10], idx.pos_by_key(Female));
    }

    #[test]
    fn create_view() {
        let mut idx = MultiPosIndex::<u8, _>::with_capacity(0);
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

        assert_eq!(1, view.min_key().unwrap());
        assert_eq!(4, view.max_key().unwrap());
    }

    mod min_max_keys {
        use super::*;

        #[test]
        fn by_insert() {
            let mut idx = MultiPosIndex::<usize>::with_capacity(0);

            // both not set
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
            assert!(idx.keys().next().is_none());

            // first insert, max and min are equal
            idx.insert(1, 0);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(1), idx.max_key());
            assert_eq!(vec![1], idx.keys().collect::<Vec<_>>());

            // first insert, max and min are equal
            idx.insert(10, 1);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(10), idx.max_key());
            assert_eq!(vec![1, 10], idx.keys().collect::<Vec<_>>());

            idx.insert(11, 2);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(11), idx.max_key());
            assert_eq!(vec![1, 10, 11], idx.keys().collect::<Vec<_>>());

            idx.delete(10, &1);
            idx.delete(11, &2);
            assert_eq!(Some(1), idx.min_key());
            assert_eq!(Some(1), idx.max_key());
            assert_eq!(vec![1], idx.keys().collect::<Vec<_>>());

            // remove last
            idx.delete(1, &0);
            assert_eq!(None, idx.min_key());
            assert_eq!(None, idx.max_key());
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
            assert_eq!(vec![1, 4], idx.keys().collect::<Vec<_>>());

            // new min
            idx.insert(0, 0);
            assert_eq!(Some(0), idx.min_key());
            assert_eq!(Some(4), idx.max_key());
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
        let mut idx = UniquePosIndex::with_capacity(5);
        idx.insert(0usize, 0);
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

        let mut idx = UniquePosIndex::<usize, Complex>::with_capacity(5);
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
