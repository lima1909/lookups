//! The `uint` is a lookup which are using the index position (the `Key`) in a `Vec` find all `Position`s.
//!
//! This lookup is well suited for consecutive numbers, which starts by `0` or `1`.
//! One use case for this conditions are primary keys (e.g. artificially created).
//!
//! ## Advantages:
//! - the finding of an `Key` is very fast (you can __directly__ jump to the value (`Key`))
//! - the `Key`s are sorted (so you get the possibility to use BitAnd and BitOr for a `Vec` for the `Key`)
//!
use crate::lookup::store::{KeyPosition, Lookup, MultiKeyPositon, Store, UniqueKeyPositon};
use std::marker::PhantomData;

/// Implementation for a `UIntLookup` with unique `Position`.
pub type UniqueUIntLookup<K = usize, X = usize> = UIntLookup<UniqueKeyPositon<X>, K, X>;
/// Implementation for a `UIntLookup` with multi `Position`s.
pub type MultiUIntLookup<K = usize, X = usize> = UIntLookup<MultiKeyPositon<X>, K, X>;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
pub struct UIntLookup<P, K = usize, X = usize> {
    inner: Vec<Option<(K, P)>>,
    max_index: usize,
    _x: PhantomData<X>,
}

impl<P, K, X> Lookup<K> for UIntLookup<P, K, X>
where
    K: Into<usize>,
    P: KeyPosition<X>,
{
    type Pos = X;
    type Extension<'a> = UIntLookupExt<'a, P, K, X>
    where
        Self: 'a;

    fn ext(&self) -> Self::Extension<'_> {
        UIntLookupExt(self)
    }

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

impl<P, K, X> Store for UIntLookup<P, K, X>
where
    K: Into<usize> + Clone,
    P: KeyPosition<X>,
{
    type Key = K;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        let idx = key.clone().into();

        // if necessary (len <= idx), than extend the vec
        self.inner.extend((self.inner.len()..=idx).map(|_| None));

        // insert new key and pos
        match self.inner.get_mut(idx) {
            Some(Some((_, p))) => p.add_pos(pos),
            _ => self.inner[idx] = Some((key, P::new(pos))),
        }

        // define new max index
        if self.max_index < idx {
            self.max_index = idx
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
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::from_iter((0..=capacity).map(|_| None)),
            max_index: 0,
            _x: PhantomData,
        }
    }
}

/// Implementation for extending the [`Lookup`].
///
pub struct UIntLookupExt<'a, P, K = usize, X = usize>(&'a UIntLookup<P, K, X>);

/// The `UIntLookupExt` can be not the fastest.
/// It depends, on how much gaps are between thes `Key`s.
///
impl<'a, P, K, X> UIntLookupExt<'a, P, K, X>
where
    K: Clone,
{
    /// Returns all stored `Key`s.
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.0.inner[..=self.0.max_index]
            .iter()
            .filter_map(|o| o.as_ref().map(|(key, _)| key.clone()))
    }

    /// Returns smallest stored `Key`.
    pub fn min_key(&self) -> Option<K> {
        self.0
            .inner
            .iter()
            .find_map(|o| o.as_ref().map(|(k, _)| k.clone()))
    }

    /// Returns greatest stored `Key`.
    pub fn max_key(&self) -> Option<K> {
        self.0
            .inner
            .get(self.0.max_index)?
            .as_ref()
            .map(|(k, _)| k.clone())
    }
}

//
// ----------- internal (private) helper implementation --------------------------
//
impl<P, K, X> UIntLookup<P, K, X> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    mod min_max {
        use super::*;

        #[test]
        fn by_insert() {
            let mut idx = MultiUIntLookup::<usize, usize>::with_capacity(0);

            // both not set
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert_eq!(0, idx.max_index);

            // first insert, max and min are equal
            idx.insert(1, 0);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());
            assert_eq!(1, idx.max_index);

            // first insert, max and min are equal
            idx.insert(10, 1);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(10), idx.ext().max_key());
            assert_eq!(10, idx.max_index);

            idx.insert(11, 2);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(11), idx.ext().max_key());
            assert_eq!(11, idx.max_index);

            idx.delete(10, &1);
            idx.delete(11, &2);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());
            assert_eq!(1, idx.max_index);

            // remove last
            idx.delete(1, &0);
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
            assert_eq!(0, idx.max_index);
        }

        #[test]
        fn by_insertwith_capacity() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            // both not set
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());

            // first insert, max and min are equal
            idx.insert(1, 1);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(1), idx.ext().max_key());

            // min and max are different
            idx.insert(4, 4);
            assert_eq!(Some(1), idx.ext().min_key());
            assert_eq!(Some(4), idx.ext().max_key());

            // new min
            idx.insert(0, 0);
            assert_eq!(Some(0), idx.ext().min_key());
            assert_eq!(Some(4), idx.ext().max_key());

            // new max
            idx.insert(6, 6);
            assert_eq!(Some(0), idx.ext().min_key());
            assert_eq!(Some(6), idx.ext().max_key());
        }

        #[test]
        fn by_delete() {
            let mut idx = MultiUIntLookup::<usize, usize>::with_capacity(5);

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

            // remove min key
            idx.delete(1, &1);
            assert_eq!(Some(2), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());

            // remove no max and no min key
            idx.delete(4, &4);
            assert_eq!(Some(2), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());

            // remove min key
            idx.delete(2, &2);
            assert_eq!(Some(3), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());

            // invalid pos, no key is removed
            idx.delete(3, &3);
            assert_eq!(Some(3), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());

            // remove last key for pos 2
            idx.delete(3, &2);
            assert_eq!(Some(5), idx.ext().min_key());
            assert_eq!(Some(5), idx.ext().max_key());

            // remove last key
            idx.delete(5, &5);
            assert_eq!(None, idx.ext().min_key());
            assert_eq!(None, idx.ext().max_key());
        }

        #[test]
        fn by_insert_and_delete() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            idx.insert(1, 1);
            idx.insert(3, 3);

            idx.delete(3, &3);
            idx.delete(1, &1);

            idx.insert(2, 2);
            idx.insert(3, 3);
        }
    }

    #[test]
    fn store_and_lookup() {
        let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
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
