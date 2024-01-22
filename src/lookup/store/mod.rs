//! The `store` module contains the interfaces for storing and accessing the lookups.
//!
pub mod position;

pub use crate::lookup::Itemer;
pub use position::{KeyPosition, MultiKeyPositon, UniqueKeyPositon};

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

/// Lookup for `Key`s. This a base Trait for more retrieval implementations.
/// Returns the positions for the searching `Key`, which the `Store` contains.
///
pub trait Lookup<Q> {
    type Pos;
    type Extension<'a>
    where
        Self: 'a;

    /// Is an extension for the `Lookup` implementation.
    /// The `Extiesion` provides `Lookup` specific capability.
    fn ext(&self) -> Self::Extension<'_>;

    /// Returns all known positions for a given `Key`.
    /// If the `Key` not exist, than is the slice empty.
    fn pos_by_key(&self, key: Q) -> &[Self::Pos];

    /// Check, that the given key exist.
    fn key_exist(&self, key: Q) -> bool {
        !self.pos_by_key(key).is_empty()
    }

    /// Returns all known positions for a given iterator of `Key`s.
    ///
    /// Hint: If the input list contains a `Key` more than ones, than containts the result list
    /// the positions also more than ones.
    fn pos_by_many_keys<'k, K>(
        &'k self,
        keys: K,
    ) -> Positions<'k, Self, Q, <K as IntoIterator>::IntoIter>
    where
        K: IntoIterator<Item = Q> + 'k,
        Self: Sized,
    {
        Positions::new(self, keys.into_iter())
    }
}

/// `Positions` is an `Iterator` for the result from [`Lookup::pos_by_many_keys()`].
pub struct Positions<'a, L: Lookup<Q>, Q, Keys> {
    lookup: &'a L,
    keys: Keys,
    pos_iter: std::slice::Iter<'a, L::Pos>,
}

impl<'a, L, Q, Keys> Positions<'a, L, Q, Keys>
where
    L: Lookup<Q>,
    Keys: Iterator<Item = Q> + 'a,
{
    pub(crate) fn new(lookup: &'a L, mut keys: Keys) -> Self {
        let pos_iter = match keys.next() {
            Some(k) => lookup.pos_by_key(k).iter(),
            None => [].iter(),
        };

        Self {
            lookup,
            keys,
            pos_iter,
        }
    }
}

impl<'a, L, Q, Keys> Iterator for Positions<'a, L, Q, Keys>
where
    L: Lookup<Q> + 'a,
    Keys: Iterator<Item = Q> + 'a,
    Self: 'a,
{
    type Item = &'a L::Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.pos_iter.next() {
            return Some(idx);
        }

        loop {
            let next_key = self.keys.next()?;
            let next_pos = self.lookup.pos_by_key(next_key);
            if !next_pos.is_empty() {
                self.pos_iter = next_pos.iter();
                return self.pos_iter.next();
            }
        }
    }
}

// ---- internal trait to create a Store by a given Iterator --------------
pub(crate) trait ToStore<'a, Item, Pos> {
    fn to_store<S, F>(self, field: F) -> S
    where
        S: Store<Pos = Pos>,
        F: Fn(&Item) -> S::Key;
}

impl<'a, It, I: 'a, Pos> ToStore<'a, I, Pos> for It
where
    It: Iterator<Item = (Pos, &'a I)> + ExactSizeIterator,
{
    fn to_store<S, F>(self, field: F) -> S
    where
        S: Store<Pos = Pos>,
        F: Fn(&I) -> S::Key,
    {
        let mut store = S::with_capacity(self.len());

        self.for_each(|(pos, item)| {
            store.insert(field(item), pos);
        });

        store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::store::{KeyPosition, MultiKeyPositon, UniqueKeyPositon};
    use rstest::rstest;
    use std::{borrow::Borrow, collections::HashMap, hash::Hash};

    struct MapIndex<K, X: KeyPosition<usize>> {
        idx: HashMap<K, X>,
    }

    impl MapIndex<String, UniqueKeyPositon<usize>> {
        fn new() -> Self {
            let mut idx = HashMap::new();
            idx.insert("a".into(), UniqueKeyPositon::new(0));
            idx.insert("b".into(), UniqueKeyPositon::new(1));
            idx.insert("c".into(), UniqueKeyPositon::new(2));
            idx.insert("s".into(), UniqueKeyPositon::new(4));
            Self { idx }
        }
    }

    impl<X: KeyPosition<usize>> MapIndex<&str, X> {
        fn from_vec(l: Vec<&'static str>) -> Self {
            let mut idx = HashMap::<&str, X>::new();

            l.into_iter()
                .enumerate()
                .for_each(|(p, s)| match idx.get_mut(s) {
                    Some(x) => {
                        x.add_pos(p);
                    }
                    None => {
                        idx.insert(s, X::new(p));
                    }
                });

            Self { idx }
        }
    }

    impl<Q, K, X: KeyPosition<usize>> Lookup<&Q> for MapIndex<K, X>
    where
        K: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + ?Sized,
    {
        type Pos = usize;
        type Extension<'a> = ()
        where
            Self: 'a;

        fn ext(&self) -> Self::Extension<'_> {}

        fn pos_by_key(&self, key: &Q) -> &[Self::Pos] {
            match self.idx.get(key) {
                Some(i) => i.as_slice(),
                None => &[],
            }
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
        let map = MapIndex::<&str, UniqueKeyPositon>::from_vec(items);
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
        let map = MapIndex::<&str, MultiKeyPositon>::from_vec(items);
        assert_eq!(expected, map.pos_by_many_keys(keys).collect::<Vec<_>>());
    }
}
