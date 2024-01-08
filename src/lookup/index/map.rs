use crate::lookup::{KeyPosition, Lookup, MultiKeyPositon, Store, UniqueKeyPositon};
use std::{borrow::Borrow, hash::Hash, marker::PhantomData};

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

/// Implementation for a `MapIndex` with unique `Position`.
pub type UniqueMapIndex<K = String, X = usize> = MapIndex<UniqueKeyPositon<X>, K, X>;
/// Implementation for a `MapIndex` with multi `Position`s.
pub type MultiMapIndex<K = String, X = usize> = MapIndex<MultiKeyPositon<X>, K, X>;

/// MapIndex is an implementation for an hash index.
///
#[derive(Debug)]
#[repr(transparent)]
pub struct MapIndex<P: KeyPosition<X>, K = String, X = usize>(HashMap<K, P>, PhantomData<X>);

impl<P, K, X, Q> Lookup<&Q> for MapIndex<P, K, X>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
    P: KeyPosition<X>,
{
    type Pos = X;

    fn key_exist(&self, key: &Q) -> bool {
        self.0.contains_key(key)
    }

    fn pos_by_key(&self, key: &Q) -> &[Self::Pos] {
        match self.0.get(key) {
            Some(p) => p.as_slice(),
            None => &[],
        }
    }
}

impl<P, K, X> Store for MapIndex<P, K, X>
where
    K: Hash + Eq,
    P: KeyPosition<X>,
{
    type Key = K;
    type Pos = X;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        match self.0.get_mut(&key) {
            Some(p) => p.add_pos(pos),
            None => {
                self.0.insert(key, P::new(pos));
            }
        }
    }

    fn delete(&mut self, key: Self::Key, pos: &Self::Pos) {
        if let Some(rm_idx) = self.0.get_mut(&key) {
            if rm_idx.remove_pos(pos) {
                self.0.remove(&key);
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        MapIndex(HashMap::with_capacity(capacity), PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter() {
        let mut idx = UniqueMapIndex::with_capacity(5);
        idx.insert(String::from("a"), 0);
        idx.insert(String::from("b"), 1);
        idx.insert(String::from("c"), 2);
        idx.insert(String::from("s"), 4);

        assert!(idx.key_exist("a"));
        assert!(!idx.key_exist("zz"));

        assert_eq!(&[1], idx.pos_by_key("b"));
        assert_eq!(&[2], idx.pos_by_key("c"));
        assert_eq!(&[1usize; 0], idx.pos_by_key("zz"));

        // check many keys
        assert_eq!(
            vec![&0, &1, &4],
            idx.pos_by_many_keys(["a", "b", "-", "s"])
                .collect::<Vec<_>>()
        );
    }
}
