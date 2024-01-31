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
    KeyPosition, Lookup, LookupExt, MultiKeyPositon, Store, UniqueKeyPositon,
};
use std::marker::PhantomData;

/// Implementation for a `Index` with unique `Position`.
pub type UniqueIndexLookup<K = usize, X = usize> = IndexLookup<UniqueKeyPositon<X>, K, X>;
/// Implementation for a `Index` with multi `Position`s.
pub type MultiIndexLookup<K = usize, X = usize> = IndexLookup<MultiKeyPositon<X>, K, X>;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
pub struct IndexLookup<P, K = usize, X = usize> {
    inner: Vec<Option<(K, P)>>,
    min_index: usize,
    max_index: usize,
    _x: PhantomData<X>,
}

impl<P, K, X> Lookup<K> for IndexLookup<P, K, X>
where
    K: Into<usize>,
    P: KeyPosition<X>,
{
    type Pos = X;

    fn key_exist(&self, key: K) -> bool {
        matches!(self.inner.get(key.into()), Some(Some(_)))
    }
    fn pos_by_key(&self, key: K) -> &[Self::Pos] {
        match self.inner.get(key.into()) {
            Some(Some((_, p))) => p.as_slice(),
            _ => &[],
        }
    }
}

impl<P, K, X> LookupExt for IndexLookup<P, K, X> {
    type Extension<'a> = IndexLookupExt<'a, P, K, X>
    where
        Self: 'a;

    fn ext(&self) -> Self::Extension<'_> {
        IndexLookupExt(self)
    }
}

impl<P, K, X> Store for IndexLookup<P, K, X>
where
    K: Into<usize> + Clone,
    P: KeyPosition<X> + Clone,
{
    type Key = K;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        let idx = key.clone().into();

        // if necessary (len <= idx), than double the vec
        if self.inner.len() <= idx {
            const PRE_ALLOC_SIZE: usize = 100;
            self.inner.resize(idx + PRE_ALLOC_SIZE, None);
        }

        // insert new key and pos
        match self.inner.get_mut(idx) {
            Some(Some((_, p))) => p.add_pos(pos),
            _ => self.inner[idx] = Some((key, P::new(pos))),
        }

        // define new max index
        if self.max_index < idx {
            self.max_index = idx;
        }
        // define new min index
        if self.min_index > idx {
            self.min_index = idx;
        }
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        let idx = key.into();

        if let Some(Some((_, rm_idx))) = self.inner.get_mut(idx) {
            if rm_idx.remove_pos(pos) {
                // las pos was deleted
                self.inner[idx] = None;

                // define new max index
                if self.max_index == idx {
                    self.max_index = self.find_new_max_index()
                }
                if self.min_index == idx {
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

/// Implementation for extending the [`Lookup`].
///
pub struct IndexLookupExt<'a, P, K, X>(&'a IndexLookup<P, K, X>);

/// The `IndexExt` can be not the fastest.
/// It depends, on how much gaps are between thes `Key`s.
///
impl<'a, P, K, X> IndexLookupExt<'a, P, K, X>
where
    K: Clone,
{
    /// Returns all stored `Key`s.
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        if self.0.min_index > self.0.max_index {
            &[]
        } else {
            &self.0.inner[self.0.min_index..=self.0.max_index]
        }
        .iter()
        .filter_map(|o| o.as_ref().map(|(key, _)| key.clone()))
    }

    /// Returns smallest stored `Key`.
    pub fn min_key(&self) -> Option<K> {
        self.0.get_key_by_index(self.0.min_index)
    }

    /// Returns greatest stored `Key`.
    pub fn max_key(&self) -> Option<K> {
        self.0.get_key_by_index(self.0.max_index)
    }
}

//
// ----------- internal (private) helper implementation --------------------------
//
impl<P, K, X> IndexLookup<P, K, X> {
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
    pub fn get_key_by_index(&self, index: usize) -> Option<K>
    where
        K: Clone,
    {
        self.inner.get(index)?.as_ref().map(|(k, _)| k.clone())
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
            fn from(gender: Gender) -> Self {
                match gender {
                    Gender::Female => 0,
                    Gender::Male => 1,
                    Gender::None => 2,
                }
            }
        }

        use Gender::*;

        let mut idx = MultiIndexLookup::<Gender, _>::with_capacity(10);

        idx.insert(Female, 10);
        idx.insert(Female, 2);
        idx.insert(Male, 1);
        idx.insert(None, 0);

        assert_eq!(Some(Female), idx.ext().min_key());
        assert_eq!(Some(None), idx.ext().max_key());

        assert_eq!(
            vec![Female, Male, None],
            idx.ext().keys().collect::<Vec<_>>()
        );

        assert_eq!(&[2, 10], idx.pos_by_key(Female));
    }

    mod min_max_keys {
        use super::*;

        #[test]
        fn by_insert() {
            let mut idx = MultiIndexLookup::<usize, usize>::with_capacity(0);

            // both not set
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert_eq!(usize::MAX, idx.min_index);
            assert_eq!(0, idx.max_index);
            assert!(idx.ext().keys().next().is_none());

            // first insert, max and min are equal
            idx.insert(1, 0);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(1, idx.max_index);
            assert_eq!(vec![1], idx.ext().keys().collect::<Vec<_>>());

            // first insert, max and min are equal
            idx.insert(10, 1);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(10), idx.ext().max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(10, idx.max_index);
            assert_eq!(vec![1, 10], idx.ext().keys().collect::<Vec<_>>());

            idx.insert(11, 2);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(11), idx.ext().max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(11, idx.max_index);
            assert_eq!(vec![1, 10, 11], idx.ext().keys().collect::<Vec<_>>());

            idx.delete(10, &1);
            idx.delete(11, &2);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(1, idx.max_index);
            assert_eq!(vec![1], idx.ext().keys().collect::<Vec<_>>());

            // remove last
            idx.delete(1, &0);
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert_eq!(usize::MAX, idx.min_index);
            assert_eq!(0, idx.max_index);
            assert!(idx.ext().keys().next().is_none());
        }

        #[test]
        fn by_insertwith_capacity() {
            let mut idx = MultiIndexLookup::<usize, usize>::with_capacity(5);
            // both not set
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert!(idx.ext().keys().next().is_none());

            // first insert, max and min are equal
            idx.insert(1, 1);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());
            assert_eq!(vec![1], idx.ext().keys().collect::<Vec<_>>());

            // min and max are different
            idx.insert(4, 4);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(4), idx.ext().max_key());
            assert_eq!(1, idx.min_index);
            assert_eq!(4, idx.max_index);
            assert_eq!(vec![1, 4], idx.ext().keys().collect::<Vec<_>>());

            // new min
            idx.insert(0, 0);
            assert_eq!(Some(0), idx.ext().min_key());
            assert_eq!(Some(4), idx.ext().max_key());
            assert_eq!(0, idx.min_index);
            assert_eq!(4, idx.max_index);
            assert_eq!(vec![0, 1, 4], idx.ext().keys().collect::<Vec<_>>());

            // new max
            idx.insert(6, 6);
            assert_eq!(Some(0), idx.ext().min_key());
            assert_eq!(Some(6), idx.ext().max_key());
            assert_eq!(vec![0, 1, 4, 6], idx.ext().keys().collect::<Vec<_>>());
        }

        #[test]
        fn by_delete() {
            let mut idx = MultiIndexLookup::<usize, usize>::with_capacity(5);

            // min/max not set
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());

            // remove not exist key/pos pair
            idx.delete(1, &1);
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());

            idx.insert(1, 1);
            idx.insert(2, 2);
            idx.insert(3, 2);
            idx.insert(4, 4);
            idx.insert(5, 5);
            assert_eq!(vec![1, 2, 3, 4, 5], idx.ext().keys().collect::<Vec<_>>());

            // remove min key
            idx.delete(1, &1);
            assert_eq!(Some(2), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());
            assert_eq!(vec![2, 3, 4, 5], idx.ext().keys().collect::<Vec<_>>());

            // remove no max and no min key
            idx.delete(4, &4);
            assert_eq!(Some(2), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());
            assert_eq!(vec![2, 3, 5], idx.ext().keys().collect::<Vec<_>>());

            // remove min key
            idx.delete(2, &2);
            assert_eq!(Some(3), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());
            assert_eq!(vec![3, 5], idx.ext().keys().collect::<Vec<_>>());

            // invalid pos, no key is removed
            idx.delete(3, &3);
            assert_eq!(Some(3), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());
            assert_eq!(vec![3, 5], idx.ext().keys().collect::<Vec<_>>());

            // remove last key for pos 2
            idx.delete(3, &2);
            assert_eq!(Some(5), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());
            assert_eq!(vec![5], idx.ext().keys().collect::<Vec<_>>());

            // remove last key
            idx.delete(5, &5);
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert!(idx.ext().keys().collect::<Vec<_>>().is_empty());
        }
    }

    #[test]
    fn store_and_lookup() {
        let mut idx = UniqueIndexLookup::<usize, usize>::with_capacity(5);
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
}
