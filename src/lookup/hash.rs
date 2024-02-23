//! The `map` is a lookup implementation for using _hashing_: with a [`std::collections::HashMap`] or [hashbrown::HashMap](https://crates.io/crates/hashbrown) (feature = "hashbrown").
//!
//! ## Advantages:
//! - all advantages, which has a hashing procedure
//!
use crate::lookup::store::{
    KeyPosition, KeyPositionAsSlice, Lookup, MultiKeyPositon, Store, UniqueKeyPositon, View,
    ViewCreator,
};
use std::{borrow::Borrow, hash::Hash, ops::Deref};

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

/// Implementation for a `HashLookup` with unique `Position`.
pub type UniquePosHash<K = String, X = usize> = HashLookup<K, UniqueKeyPositon<X>>;
/// Implementation for a `HashLookup` with multi `Position`s.
pub type MultiPosHash<K = String, X = usize> = HashLookup<K, MultiKeyPositon<X>>;

/// `HashLookup` is an implementation for an hash index.
///
#[derive(Debug)]
#[repr(transparent)]
pub struct HashLookup<K, P>(HashMap<K, P>);

impl<Q, K, P> Lookup<&Q> for HashLookup<K, P>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
    P: KeyPositionAsSlice,
{
    type Pos = P::Pos;

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

impl<'a, K, P> ViewCreator<'a> for HashLookup<K, P>
where
    K: Hash + Eq + Clone,
    P: KeyPositionAsSlice + 'a,
{
    type Key = K;
    type Lookup = HashLookup<K, &'a P>;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Lookup>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut map = HashMap::<K, &P>::with_capacity(self.0.len());

        for key in keys {
            if let Some(p) = self.0.get(&key) {
                map.insert(key.clone(), p);
            }
        }

        View::new(HashLookup(map))
    }
}

impl<K, P> Store for HashLookup<K, P>
where
    K: Hash + Eq,
    P: KeyPosition,
{
    type Key = K;
    type Pos = P::Pos;

    fn insert(&mut self, key: Self::Key, pos: Self::Pos) {
        match self.0.get_mut(&key) {
            Some(p) => p.add_pos(pos),
            None => {
                self.0.insert(key, P::from_pos(pos));
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
        HashLookup(HashMap::with_capacity(capacity))
    }
}

/// Implementation for extending the [`Lookup`].
#[repr(transparent)]
pub struct HashLookupExt<K, P>(HashLookup<K, P>);

impl<K, P> Deref for HashLookup<K, P> {
    type Target = HashLookupExt<K, P>;

    fn deref(&self) -> &Self::Target {
        // SAFTY:
        // self is a valid pointer and
        // HashLookupExt is repr(transparent) thus has the same memory layout like HashLookup
        unsafe { &*(self as *const HashLookup<K, P> as *const HashLookupExt<K, P>) }
    }
}

impl<K, P> HashLookupExt<K, P> {
    pub fn keys(&self) -> impl Iterator<Item = &'_ K> {
        self.0 .0.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_view() {
        let mut idx = UniquePosHash::with_capacity(5);
        idx.insert(String::from("a"), 0);
        idx.insert(String::from("b"), 1);
        idx.insert(String::from("c"), 2);
        idx.insert(String::from("s"), 4);

        assert!(idx.key_exist("a"));

        let view = idx.create_view([String::from("b"), String::from("s")]);
        assert!(!view.key_exist("a"));
        assert!(!view.key_exist("x"));

        assert!(view.key_exist("b"));
        assert!(view.key_exist("s"));

        assert_eq!(&[4], view.pos_by_key("s"));
        assert_eq!(&[1usize; 0], view.pos_by_key("c"));

        assert_eq!(
            vec![&1, &4],
            view.pos_by_many_keys(["a", "b", "-", "s"])
                .collect::<Vec<_>>()
        );

        let keys = view.keys().collect::<Vec<_>>();
        assert!(keys.contains(&&String::from("b")));
        assert!(keys.contains(&&String::from("s")));
    }

    #[test]
    fn store_and_lookup() {
        let mut idx = UniquePosHash::with_capacity(5);
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
