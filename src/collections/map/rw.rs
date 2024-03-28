//! `Read write` implementations for lookup collections `HashMap`.
//!

use crate::{
    collections::{map::ro, Edit},
    lookup::store::{position::KeyPosition, Lookup, Retriever, Store},
};
use std::{hash::Hash, ops::Deref};

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug)]
pub struct LkupHashMap<S, F, K, V> {
    field: F,
    inner: ro::LkupHashMap<S, K, V>,
}

impl<S, F, K, V> LkupHashMap<S, F, K, V>
where
    S: Store<Pos = K>,
    F: Fn(&V) -> S::Key,
{
    pub fn new<L, P>(lookup: L, field: F) -> Self
    where
        L: Lookup<S, P>,
        P: KeyPosition<Pos = K>,
        F: Clone,
        K: Clone,
    {
        Self {
            field: field.clone(),
            inner: ro::LkupHashMap::new(lookup, field, HashMap::new()),
        }
    }

    pub fn insert(&mut self, key: K, item: V) -> Option<V>
    where
        K: Hash + Eq + Clone,
    {
        self.inner.0.store.insert((self.field)(&item), key.clone());
        self.inner.0.items.insert(key, item)
    }
}

impl<S, F, K, V> Deref for LkupHashMap<S, F, K, V> {
    type Target = ro::LkupBaseMap<S, HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, F, K, V> Edit<K, V> for LkupHashMap<S, F, K, V>
where
    S: Store<Pos = K>,
    F: Fn(&V) -> S::Key,
    K: Hash + Eq,
{
    type Retriever = S;

    fn update<U>(&mut self, key: K, mut update: U) -> Option<&V>
    where
        U: FnMut(&mut V),
    {
        let v = self.inner.0.items.get_mut(&key)?;
        let old_key = (self.field)(v);
        update(v);
        self.inner.0.store.update(old_key, key, (self.field)(v));
        Some(v)
    }

    fn remove(&mut self, key: K) -> Option<V> {
        let removed = self.inner.0.items.remove(&key)?;
        self.inner.0.store.delete((self.field)(&removed), &key);
        Some(removed)
    }

    fn get_indices_by_key<Q>(&self, key: Q) -> &[K]
    where
        Self::Retriever: Retriever<Q, Pos = K>,
    {
        self.inner.store.pos_by_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HashLookup, IndexLookup};

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    #[test]
    fn map_key_string() {
        let mut m = LkupHashMap::new(IndexLookup::with_multi_keys(), |c: &Car| c.0);
        m.insert(String::from("Audi"), Car(99, String::from("Audi")));
        m.insert(String::from("BMW"), Car(1, String::from("BMW")));

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));

        // update
        assert_eq!(
            Some(&Car(1_000, String::from("BMW"))),
            m.update(String::from("BMW"), |c| c.0 = 1_000)
        );
        assert!(m.lkup().contains_key(1_000));

        assert_eq!(None, m.update("NotFound".into(), |_c| {}));

        // update by lookup-key
        assert_eq!(1, m.update_by_key(1_000, |c| c.0 = 1));
        assert_eq!(0, m.update_by_key(1_000, |c| c.0 = 1_000));
        assert_eq!(1, m.update_by_key(1, |c| c.0 = 1_000));

        // remove
        assert_eq!(2, m.len());
        assert_eq!(
            Some(Car(1_000, String::from("BMW"))),
            m.remove("BMW".into())
        );
        assert!(!m.contains_key("BMW"));
        assert!(!m.lkup().contains_key(1_000));
        assert_eq!(1, m.len());

        // remove by lookup-key
        assert_eq!(0, m.remove_by_key(2));
        assert_eq!(1, m.len());

        assert_eq!(1, m.remove_by_key(99));
        assert_eq!(0, m.len());
    }

    #[test]
    fn map_key_usize() {
        let mut m = LkupHashMap::new(HashLookup::with_unique_key(), |c: &Car| c.1.clone());
        m.insert(99, Car(99, String::from("Audi")));
        m.insert(1, Car(1, String::from("BMW")));

        assert!(m.contains_key(&1));

        assert!(m.lkup().contains_key("BMW"));
        assert!(!m.lkup().contains_key("NOT FOUND"));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));

        // update
        assert_eq!(
            Some(&Car(1, String::from("VW"))),
            m.update(1, |c| c.1 = String::from("VW"))
        );
        assert!(m.lkup().contains_key("VW"));

        assert_eq!(None, m.update(1_000, |_c| {}));

        // remove
        assert_eq!(2, m.len());
        assert_eq!(Some(Car(1, String::from("VW"))), m.remove(1));
        assert!(!m.contains_key(&1));
        assert!(!m.lkup().contains_key("VW"));
        assert_eq!(1, m.len());
    }
}
