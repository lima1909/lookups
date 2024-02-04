//! The `map` is a lookup implementation for using _hashing_: with a [`std::collections::HashMap`] or [hashbrown::HashMap](https://crates.io/crates/hashbrown) (feature = "hashbrown").
//!
//! ## Advantages:
//! - all advantages, which has a hashing procedure
//!
use crate::{
    lookup::store::{KeyPosition, Lookup, MultiKeyPositon, Store, UniqueKeyPositon},
    HashMap,
};
use std::{borrow::Borrow, hash::Hash, marker::PhantomData, ops::Deref};

/// Implementation for a `HashLookup` with unique `Position`.
pub type UniqueHash<K = String, X = usize> = HashLookup<UniqueKeyPositon<X>, K, X>;
/// Implementation for a `HashLookup` with multi `Position`s.
pub type MultiHash<K = String, X = usize> = HashLookup<MultiKeyPositon<X>, K, X>;

/// `HashLookup` is an implementation for an hash index.
///
#[derive(Debug)]
#[repr(transparent)]
pub struct HashLookup<P: KeyPosition<X>, K = String, X = usize>(HashMap<K, P>, PhantomData<X>);

impl<P, K, X, Q> Lookup<&Q> for HashLookup<P, K, X>
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

impl<P, K, X> Store for HashLookup<P, K, X>
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
        HashLookup(HashMap::with_capacity(capacity), PhantomData)
    }
}

/// Implementation for extending the [`Lookup`].
#[repr(transparent)]
pub struct HashLookupExt<P: KeyPosition<X>, K, X>(HashLookup<P, K, X>);

impl<P: KeyPosition<X>, K, X> Deref for HashLookup<P, K, X> {
    type Target = HashLookupExt<P, K, X>;

    fn deref(&self) -> &Self::Target {
        // SAFTY:
        // self is a valid pointer and
        // HashLookupExt is repr(transparent) thus has the same memory layout like HashLookup
        unsafe { &*(self as *const HashLookup<P, K, X> as *const HashLookupExt<P, K, X>) }
    }
}

impl<P: KeyPosition<X>, K, X> HashLookupExt<P, K, X> {
    pub fn keys(&self) -> impl Iterator<Item = &'_ K> {
        self.0 .0.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_lookup() {
        let mut idx = UniqueHash::with_capacity(5);
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

        use std::collections::HashSet;

        let keys = idx.keys().cloned().collect::<HashSet<_>>();
        assert_eq!(4, keys.len());
        assert!(keys.contains("a"));
        assert!(keys.contains("b"));
        assert!(keys.contains("c"));
        assert!(keys.contains("s"));
    }
}
