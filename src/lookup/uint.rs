//! The `uint` is a lookup which are using the index position (the `Key`) in a `Vec` find all `Position`s.
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
    inner: Vec<P>,
    min_idx: Option<usize>,
    max_idx: Option<usize>,
    _key: PhantomData<K>,
    _pos: PhantomData<X>,
}

impl<P, K, X> Lookup<K> for UIntLookup<P, K, X>
where
    K: Into<usize>,
    P: KeyPosition<X>,
{
    type Pos = X;
    type Extension<'a> = UIntLookupExt<'a, P, K, X> where P:'a, K:'a, X: 'a;

    fn extension(&self) -> Self::Extension<'_> {
        UIntLookupExt(self)
    }

    fn pos_by_key(&self, key: K) -> &[Self::Pos] {
        match self.inner.get(key.into()) {
            Some(p) => p.as_slice(),
            None => &[],
        }
    }
}

impl<P, K, X> Store for UIntLookup<P, K, X>
where
    K: Into<usize>,
    P: KeyPosition<X> + Clone,
{
    type Key = K;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        let idx = key.into();

        if self.inner.len() <= idx {
            let l = if idx == 0 { 2 } else { idx * 2 };
            self.inner.resize(l, P::none());
        }

        match self.inner.get_mut(idx) {
            Some(p) => p.add_pos(pos),
            None => self.inner[idx] = P::new(pos),
        }

        self.insert_min_max(idx);
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        let idx = key.into();

        if let Some(rm_idx) = self.inner.get_mut(idx) {
            if rm_idx.remove_pos(pos) {
                self.inner[idx] = P::none();
                self.delete_min_max(idx);
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            min_idx: None,
            max_idx: None,
            _key: PhantomData,
            _pos: PhantomData,
        }
    }
}

/// Implementation for extending the [`Lookup`].
///
pub struct UIntLookupExt<'a, P, K = usize, X = usize>(&'a UIntLookup<P, K, X>);

impl<'a, P, K, X> UIntLookupExt<'a, P, K, X> {
    pub fn key_indexes(&self) -> impl Iterator<Item = usize> + 'a
    where
        P: KeyPosition<X>,
    {
        self.0.key_iter().filter_map(not_none)
    }

    pub fn min_key_index(&self) -> Option<usize> {
        self.0.min_idx
    }

    pub fn max_key_index(&self) -> Option<usize> {
        self.0.max_idx
    }
}

//
// ----------- internal (private) helper implementation --------------------------
//
impl<P, K, X> UIntLookup<P, K, X> {
    fn key_iter(&self) -> impl Iterator<Item = (usize, &'_ P)> {
        let min = self.min_idx.unwrap_or_default();
        let max = self.max_idx.unwrap_or_default();

        self.inner[min..=max].iter().enumerate()
    }

    fn insert_min_max(&mut self, new_value: usize) {
        match (self.min_idx, self.max_idx) {
            (None, None) => {
                self.min_idx = Some(new_value);
                self.max_idx = Some(new_value);
            }
            (Some(min), Some(max)) => {
                if new_value < min {
                    self.min_idx = Some(new_value)
                } else if new_value > max {
                    self.max_idx = Some(new_value)
                }
            }
            (None, Some(_)) => unreachable!("max, but no min value"),
            (Some(_), None) => unreachable!("min, but no max value"),
        }
    }

    fn delete_min_max(&mut self, new_value: usize)
    where
        P: KeyPosition<X>,
    {
        match (self.min_idx, self.max_idx) {
            (None, None) => {}
            (Some(min), Some(max)) => {
                if min == new_value {
                    self.min_idx = self.find_min_idx();
                }
                if max == new_value {
                    self.max_idx = self.find_max_idx();
                }
            }
            (None, Some(_)) => unreachable!("max, but no min value"),
            (Some(_), None) => unreachable!("min, but no max value"),
        }
    }

    fn find_min_idx(&self) -> Option<usize>
    where
        P: KeyPosition<X>,
    {
        self.inner.iter().enumerate().find_map(not_none)
    }

    fn find_max_idx(&self) -> Option<usize>
    where
        P: KeyPosition<X>,
    {
        self.inner.iter().enumerate().rev().find_map(|(pos, p)| {
            if p.as_slice().is_empty() {
                None
            } else {
                Some(self.inner.len() - pos)
            }
        })
    }
}

#[inline]
fn not_none<X, P: KeyPosition<X>>((pos, p): (usize, &P)) -> Option<usize> {
    if p.as_slice().is_empty() {
        None
    } else {
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod min_max {
        use super::*;

        #[test]
        fn by_insert() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);

            // both not set
            assert_eq!(None, idx.min_idx);
            assert_eq!(None, idx.max_idx);

            // first insert, max and min are equal
            idx.insert(1, 1);
            assert_eq!(Some(1), idx.min_idx);
            assert_eq!(Some(1), idx.max_idx);

            // min and max are different
            idx.insert(4, 4);
            assert_eq!(Some(1), idx.min_idx);
            assert_eq!(Some(4), idx.max_idx);

            // new min
            idx.insert(0, 0);
            assert_eq!(Some(0), idx.min_idx);
            assert_eq!(Some(4), idx.max_idx);

            // new max
            idx.insert(6, 6);
            assert_eq!(Some(0), idx.min_idx);
            assert_eq!(Some(6), idx.max_idx);
        }

        #[test]
        fn by_delete() {
            let mut idx = MultiUIntLookup::<usize, usize>::with_capacity(5);

            // // min/max not set
            // assert_eq!(None, idx.min_idx);
            // assert_eq!(None, idx.max_idx);

            // // remove not exist key/pos pair
            // idx.delete(1, &1);
            // assert_eq!(None, idx.min_idx);
            // assert_eq!(None, idx.max_idx);

            idx.insert(1, 1);
            idx.insert(2, 2);
            idx.insert(3, 2);
            idx.insert(4, 4);
            idx.insert(5, 5);

            // remove min key
            idx.delete(1, &1);
            assert_eq!(Some(2), idx.min_idx);
            assert_eq!(Some(5), idx.max_idx);

            // remove no max and no min key
            idx.delete(4, &4);
            assert_eq!(Some(2), idx.min_idx);
            assert_eq!(Some(5), idx.max_idx);

            // remove min key
            idx.delete(2, &2);
            assert_eq!(Some(3), idx.min_idx);
            assert_eq!(Some(5), idx.max_idx);

            // invalid pos, no key is removed
            idx.delete(3, &3);
            assert_eq!(Some(3), idx.min_idx);
            assert_eq!(Some(5), idx.max_idx);

            // remove last key for pos 2
            idx.delete(3, &2);
            assert_eq!(Some(5), idx.min_idx);
            assert_eq!(Some(5), idx.max_idx);

            // remove last key
            idx.delete(5, &5);
            assert_eq!(None, idx.min_idx);
            assert_eq!(None, idx.max_idx);
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
