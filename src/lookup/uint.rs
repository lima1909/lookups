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
#[repr(transparent)]
pub struct UIntLookup<P, K = usize, X = usize> {
    inner: Vec<P>,
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
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        let idx = key.into();

        if let Some(rm_idx) = self.inner.get_mut(idx) {
            if rm_idx.remove_pos(pos) {
                self.inner[idx] = P::none();
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            _key: PhantomData,
            _pos: PhantomData,
        }
    }
}

pub struct UIntLookupExt<'a, P, K = usize, X = usize>(&'a UIntLookup<P, K, X>);

impl<'a, P, K, X> UIntLookupExt<'a, P, K, X> {
    pub fn foo(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
