//! The `store` module contains the interfaces for storing and accessing the lookups.
//!
pub mod position;

use position::{KeyPosition, MultiKeyPosition, UniqueKeyPosition};

/// Retriever for `Key`s. This a base Trait for more retrieval implementations.
/// Returns the positions for the searching `Key`, which the `Store` contains.
///
pub trait Retriever<Q> {
    type Pos;

    /// Check, that the given key exist.
    fn key_exist(&self, key: Q) -> bool;

    /// Returns all known positions for a given `Key`.
    /// If the `Key` not exist, than is the slice empty.
    fn pos_by_key(&self, key: Q) -> &[Self::Pos];

    /// Returns all known positions for a given iterator of `Key`s.
    ///
    /// Hint: If the input list contains a `Key` more than ones, than containts the result list
    /// the positions also more than ones.
    fn pos_by_many_keys<'a, K>(&'a self, keys: K) -> impl Iterator<Item = &'a Self::Pos>
    where
        K: IntoIterator<Item = Q>,
        Self::Pos: 'a,
    {
        keys.into_iter().flat_map(|q| self.pos_by_key(q))
    }
}

impl<R, Q> Retriever<Q> for &R
where
    R: Retriever<Q>,
{
    type Pos = R::Pos;

    fn key_exist(&self, key: Q) -> bool {
        (*self).key_exist(key)
    }

    fn pos_by_key(&self, key: Q) -> &[Self::Pos] {
        (*self).pos_by_key(key)
    }
}

/// `Positions` create an `Iterator` for all saved positions.
pub trait Positions<'a> {
    type Pos;

    /// Returns all knwon positions as an iterator.
    fn positions(&'a self) -> impl Iterator<Item = &'a Self::Pos>;
}

/// Store is an container which the mapping between the `Key`s and they `Position`s stored.
///
pub trait Store {
    type Key;
    type Pos;

    /// Insert an `Key` with the associated `Position`s.
    ///
    fn insert(&mut self, key: Self::Key, pos: Self::Pos);

    /// Update means: `Key` changed, but `Position` stays the same.
    ///
    fn update(&mut self, old_key: Self::Key, pos: Self::Pos, new_key: Self::Key) {
        self.delete(old_key, &pos);
        self.insert(new_key, pos);
    }

    /// Delete means: if an `Key` has more than one `Position`, then remove only the given `Position`:
    /// If the `Key` not exist, then is `delete`ignored:
    ///
    fn delete(&mut self, key: Self::Key, pos: &Self::Pos);

    /// To reduce memory allocations can create an `Store` with capacity.
    ///
    fn with_capacity(capacity: usize) -> Self;
}

/// `Lookup` creates an unique or multi `Key` lookup.
pub trait Lookup<S, P>
where
    S: Store,
    P: KeyPosition,
{
    fn new() -> Self;

    // Create an `Lookup` for a given `KeyPosition` implementation.
    fn with_key<K>() -> Self
    where
        K: KeyPosition,
        Self: Lookup<S, K> + Sized,
    {
        Lookup::<S, K>::new()
    }

    // Create an `Lookup` for an unique `Key`.
    fn with_unique_key() -> Self
    where
        P::Pos: PartialEq,
        Self: Lookup<S, UniqueKeyPosition<P::Pos>> + Sized,
    {
        Lookup::<S, UniqueKeyPosition<P::Pos>>::new()
    }

    // Create an `Lookup` for multiple `Key`s.
    fn with_multi_keys() -> Self
    where
        P::Pos: Ord,
        Self: Lookup<S, MultiKeyPosition<P::Pos>> + Sized,
    {
        Lookup::<S, MultiKeyPosition<P::Pos>>::new()
    }

    /// Create a new `Store` for a `collection` from type `list` (e.g. `LkupVec`).
    /// The `Pos`-Type is always `usize`.
    fn new_list_store<'a, F, K, It, I: 'a>(&self, field: &F, it: It) -> S
    where
        It: Iterator<Item = &'a I> + ExactSizeIterator,
        F: Fn(&I) -> K,
        S: Store<Key = K, Pos = usize>,
    {
        let mut store = S::with_capacity(it.len());
        it.enumerate()
            .for_each(|(pos, item)| store.insert(field(item), pos));
        store
    }

    /// Create a new `Store` for a `collection` from type `map` (e.g. `LkupHashMap`).
    fn new_map_store<'a, F, K, It, I: 'a>(&self, field: &F, it: It) -> S
    where
        It: Iterator<Item = (&'a P::Pos, &'a I)> + ExactSizeIterator,
        F: Fn(&I) -> K,
        S: Store<Key = K, Pos = P::Pos>,
        P::Pos: Clone + 'a,
    {
        let mut store = S::with_capacity(it.len());
        it.for_each(|(pos, item)| store.insert(field(item), pos.clone()));
        store
    }
}

/// The Idea of a `View` is like database view.
/// They shows a subset of `Keys` which are saved in the [`crate::lookup::store::Store`].
pub trait ViewCreator<'a> {
    type Key;
    type Retriever;

    /// Create a `View` by the given `Key`s.
    fn create_view<It>(&'a self, keys: It) -> View<Self::Retriever>
    where
        It: IntoIterator<Item = Self::Key>;
}

/// A wrapper for a `Lookup` implementation
#[repr(transparent)]
pub struct View<R>(R);

impl<R> View<R> {
    pub fn new(retriever: R) -> Self {
        Self(retriever)
    }
}

impl<R, Q> Retriever<Q> for View<R>
where
    R: Retriever<Q>,
{
    type Pos = R::Pos;

    fn key_exist(&self, key: Q) -> bool {
        self.0.key_exist(key)
    }

    fn pos_by_key(&self, key: Q) -> &[Self::Pos] {
        self.0.pos_by_key(key)
    }
}

impl<'a, P> Positions<'a> for View<P>
where
    P: Positions<'a>,
{
    type Pos = P::Pos;

    fn positions(&'a self) -> impl Iterator<Item = &'a Self::Pos>
    where
        Self::Pos: 'a,
    {
        self.0.positions()
    }
}

impl<R> std::ops::Deref for View<R>
where
    R: std::ops::Deref,
{
    type Target = R::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::store::position::{
        KeyPosition, KeyPositionAsSlice, MultiKeyPosition, UniqueKeyPosition,
    };
    use rstest::rstest;
    use std::{borrow::Borrow, collections::HashMap, hash::Hash};

    struct MapIndex<K, P> {
        idx: HashMap<K, P>,
    }

    impl MapIndex<String, UniqueKeyPosition<usize>> {
        fn new() -> Self {
            let mut idx = HashMap::new();
            idx.insert("a".into(), UniqueKeyPosition::from_pos(0));
            idx.insert("b".into(), UniqueKeyPosition::from_pos(1));
            idx.insert("c".into(), UniqueKeyPosition::from_pos(2));
            idx.insert("s".into(), UniqueKeyPosition::from_pos(4));
            Self { idx }
        }
    }

    impl<P: KeyPosition<Pos = usize>> MapIndex<&str, P> {
        fn from_vec(l: Vec<&'static str>) -> Self {
            let mut idx = HashMap::<&str, P>::new();

            l.into_iter()
                .enumerate()
                .for_each(|(p, s)| match idx.get_mut(s) {
                    Some(x) => {
                        x.add_pos(p);
                    }
                    None => {
                        idx.insert(s, P::from_pos(p));
                    }
                });

            Self { idx }
        }
    }

    impl<Q, K, P> Retriever<&Q> for MapIndex<K, P>
    where
        K: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
        P: KeyPositionAsSlice,
    {
        type Pos = P::Pos;

        fn pos_by_key(&self, key: &Q) -> &[Self::Pos] {
            match self.idx.get(key) {
                Some(i) => i.as_position_slice(),
                None => &[],
            }
        }

        fn key_exist(&self, key: &Q) -> bool {
            self.idx.contains_key(key)
        }
    }

    #[test]
    fn filter() {
        let l = MapIndex::new();

        assert!(l.key_exist("a"));
        assert!(!l.key_exist("zz"));

        assert_eq!(&[1], l.pos_by_key("b"));
        assert_eq!(&[2], l.pos_by_key("c"));
        assert_eq!(&[1usize; 0], l.pos_by_key("zz"));

        // check many keys
        assert_eq!(
            vec![&0, &1, &4],
            l.pos_by_many_keys(["a", "b", "-", "s"]).collect::<Vec<_>>()
        );
    }

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::one_found(vec!["c"], vec![&3])]
    #[case::one_doble_found(vec!["c", "c"], vec![&3, &3])] // double key create double positions
    #[case::one_not_found(vec!["-"], vec![])]
    #[case::m_z_a(vec!["m", "z", "a"], vec![&5, &1])]
    #[case::a_m_z(vec![ "a","m", "z"], vec![&1, &5])]
    #[case::z_m_a(vec![ "z","m", "a"], vec![&5, &1])]
    #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![&5, &1])]
    #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![&5, &1])]
    fn iter_unique_positions(#[case] keys: Vec<&str>, #[case] expected: Vec<&usize>) {
        let items = vec!["x", "a", "b", "c", "y", "z"];
        let map = MapIndex::<&str, UniqueKeyPosition<usize>>::from_vec(items);
        assert_eq!(expected, map.pos_by_many_keys(keys).collect::<Vec<_>>());
    }

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::one_found(vec!["c"], vec![&3])]
    #[case::two_found(vec!["x"], vec![&0, &4])]
    #[case::two_double_found(vec!["x", "x"], vec![&0, &4, &0, &4])] // double key create double positions
    #[case::one_not_found(vec!["-"], vec![])]
    #[case::m_z_a(vec!["m", "z", "a"], vec![&6, &1])]
    #[case::a_m_z(vec![ "a","m", "z"], vec![&1, &6])]
    #[case::z_m_a(vec![ "z","m", "a"], vec![&6, &1])]
    #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![&6, &1])]
    #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![&6, &1])]
    #[case::double_x(vec!["x"], vec![&0, &4])]
    #[case::a_double_x(vec!["a", "x"], vec![&1, &0, &4])]
    fn iter_multi_positions(#[case] keys: Vec<&str>, #[case] expected: Vec<&usize>) {
        let items = vec!["x", "a", "b", "c", "x", "y", "z"];
        let map = MapIndex::<&str, MultiKeyPosition<usize>>::from_vec(items);
        assert_eq!(expected, map.pos_by_many_keys(keys).collect::<Vec<_>>());
    }
}
