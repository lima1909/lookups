//! The `uint` is a lookup which are using the index position (the `Key`) in a `Vec` find all `Position`s.
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
    min_idx: Option<usize>,
    max_idx: Option<usize>,
    _x: PhantomData<X>,
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
    P: KeyPosition<X> + Clone,
{
    type Key = K;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        let idx = key.clone().into();

        if self.inner.len() <= idx {
            let l = if idx == 0 { 2 } else { idx * 2 };
            self.inner.resize(l, None);
        }

        match self.inner.get_mut(idx) {
            Some(Some((_, p))) => p.add_pos(pos),
            _ => self.inner[idx] = Some((key, P::new(pos))),
        }

        self.insert_min_max(idx);
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        let idx = key.clone().into();

        if let Some(Some((_, rm_idx))) = self.inner.get_mut(idx) {
            if rm_idx.remove_pos(pos) {
                self.inner[idx] = None;
                self.delete_min_max(idx);
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            min_idx: None,
            max_idx: None,
            _x: PhantomData,
        }
    }
}

/// Implementation for extending the [`Lookup`].
///
pub struct UIntLookupExt<'a, P, K = usize, X = usize>(&'a UIntLookup<P, K, X>);

impl<'a, P, K, X> UIntLookupExt<'a, P, K, X> {
    pub fn keys(&self) -> impl Iterator<Item = K> + '_
    where
        K: Clone,
    {
        self.0.values().map(|(key, _)| key.clone())
    }

    pub fn min_key(&self) -> Option<K>
    where
        K: Clone,
    {
        let idx = self.0.min_idx?;
        let pair = self.0.inner[idx].as_ref()?;
        let key = &pair.0;
        Some(key.clone())
    }

    pub fn max_key(&self) -> Option<K>
    where
        K: Clone,
    {
        let idx = self.0.max_idx?;
        let pair = self.0.inner[idx].as_ref()?;
        let key = &pair.0;
        Some(key.clone())
    }

    pub fn min_key_index(&self) -> Option<usize> {
        self.0.min_idx
    }

    pub fn max_key_index(&self) -> Option<usize> {
        self.0.max_idx
    }
}

// ----------- And | OR --------------------------------------------------------
impl<P, K, X> UIntLookup<P, K, X>
where
    P: KeyPosition<X>,
    K: Into<usize> + Clone,
{
    // Intersection is using for AND
    #[allow(dead_code)]
    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = (K, &[X])> + 'a {
        self.values().filter_map(|(key, p)| {
            if other.key_exist(key.clone()) {
                return Some((key.clone(), p.as_slice()));
            }

            None
        })
    }

    // Union is using for OR
    #[allow(dead_code)]
    pub fn union<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = (K, &[X])> + 'a {
        self.values()
            .map(|(key, p)| (key.clone(), p.as_slice()))
            .chain(other.difference(self))
    }

    // Difference are `Key`s which are in self but not in other.
    #[allow(dead_code)]
    fn difference<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = (K, &[X])> + 'a {
        self.values().filter_map(|(key, p)| {
            if !other.key_exist(key.clone()) {
                return Some((key.clone(), p.as_slice()));
            }

            None
        })
    }
}

//
// ----------- internal (private) helper implementation --------------------------
//
impl<P, K, X> UIntLookup<P, K, X> {
    fn values(&self) -> impl Iterator<Item = &(K, P)> {
        let min = self.min_idx.unwrap_or_default();
        let max = self.max_idx.unwrap_or_default();

        self.inner[min..=max].iter().filter_map(|v| v.as_ref())
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

    fn delete_min_max(&mut self, new_value: usize) {
        match (self.min_idx, self.max_idx) {
            (None, None) => {}
            (Some(min), Some(max)) => {
                if min == new_value || max == new_value {
                    self.min_idx = self.find_min_idx();
                    self.max_idx = self.find_max_idx();
                }
            }
            (None, Some(_)) => unreachable!("max, but no min value"),
            (Some(_), None) => unreachable!("min, but no max value"),
        }
    }

    fn find_min_idx(&self) -> Option<usize> {
        self.inner.iter().enumerate().find_map(|(pos, o)| {
            if o.is_some() {
                return Some(pos);
            }
            None
        })
    }

    fn find_max_idx(&self) -> Option<usize> {
        self.inner.iter().rev().enumerate().find_map(|(pos, o)| {
            if o.is_some() {
                return Some(self.inner.len() - pos - 1);
            }
            None
        })
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

            // min/max not set
            assert_eq!(None, idx.min_idx);
            assert_eq!(None, idx.max_idx);

            // remove not exist key/pos pair
            idx.delete(1, &1);
            assert_eq!(None, idx.min_idx);
            assert_eq!(None, idx.max_idx);

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

    mod union_inters_diff {
        use super::*;

        #[test]
        fn intersection() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            idx.insert(1, 1);
            idx.insert(3, 3);

            let mut other = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            other.insert(2, 2);
            other.insert(3, 3);
            other.insert(5, 5);

            assert_eq!(
                vec![(3usize, vec![3usize].as_slice())],
                idx.intersection(&other).collect::<Vec<_>>()
            );

            // after delete 3, the intersection is empty
            idx.delete(3, &3);
            assert_eq!(None, idx.intersection(&other).next());

            // insert new two
            idx.insert(2, 2);
            idx.insert(3, 3);

            assert_eq!(
                vec![(2usize, vec![2usize].as_slice()), (3, vec![3].as_slice())],
                idx.intersection(&other).collect::<Vec<_>>()
            );
        }

        #[test]
        fn union() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            idx.insert(1, 1);
            idx.insert(3, 3);

            let mut other = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            other.insert(2, 2);
            other.insert(3, 3);
            other.insert(5, 5);

            assert_eq!(
                vec![
                    (1usize, vec![1usize].as_slice()),
                    (3, &[3]),
                    (2, &[2]),
                    (5, &[5])
                ],
                idx.union(&other).collect::<Vec<_>>()
            );

            // after delete 3, the intersection is empty
            idx.delete(3, &3);
            assert_eq!(
                vec![
                    (1usize, vec![1usize].as_slice()),
                    (2, &[2]),
                    (3, &[3]),
                    (5, &[5])
                ],
                idx.union(&other).collect::<Vec<_>>()
            );

            // insert new two
            idx.insert(2, 2);
            idx.insert(3, 3);

            assert_eq!(
                vec![
                    (1usize, vec![1usize].as_slice()),
                    (2, &[2]),
                    (3, &[3]),
                    (5, &[5])
                ],
                idx.union(&other).collect::<Vec<_>>()
            );
        }

        #[test]
        fn difference() {
            let mut idx = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            idx.insert(1, 1);
            idx.insert(3, 3);

            let mut other = UniqueUIntLookup::<usize, usize>::with_capacity(5);
            other.insert(2, 2);
            other.insert(3, 3);
            other.insert(5, 5);

            assert_eq!(
                vec![(1usize, vec![1usize].as_slice())],
                idx.difference(&other).collect::<Vec<_>>()
            );

            // after delete 3, the difference is the same
            idx.delete(3, &3);
            assert_eq!(
                vec![(1usize, vec![1usize].as_slice())],
                idx.difference(&other).collect::<Vec<_>>()
            );

            // after delete 1, the difference is empty
            idx.delete(1, &1);
            assert_eq!(None, idx.difference(&other).next());

            // insert new two
            idx.insert(2, 2);
            idx.insert(3, 3);
            assert_eq!(None, idx.difference(&other).next());

            idx.insert(0, 0);
            idx.insert(99, 99);
            assert_eq!(
                vec![(0usize, vec![0usize].as_slice()), (99, &[99])],
                idx.difference(&other).collect::<Vec<_>>()
            );
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
