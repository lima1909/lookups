//! `Read write` implementations for lookup collections `HashMap`.
//!

use crate::{collections::map::ro, lookup::store::Store};
use std::{borrow::Borrow, hash::Hash, ops::Deref};

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug)]
pub struct LkupHashMap<S, K, V, F> {
    field: F,
    inner: ro::LkupHashMap<S, K, V>,
}

impl<S, K, V, F> LkupHashMap<S, K, V, F>
where
    S: Store<Pos = K>,
    F: Fn(&V) -> S::Key,
{
    pub fn new(field: F) -> Self
    where
        F: Clone,
        K: Clone,
    {
        Self {
            field: field.clone(),
            inner: ro::LkupHashMap::new(field, HashMap::new()),
        }
    }

    pub fn insert(&mut self, key: K, item: V) -> Option<V>
    where
        K: Hash + Eq + Clone,
    {
        self.inner.0.store.insert((self.field)(&item), key.clone());
        self.inner.0.items.insert(key, item)
    }

    pub fn update<Q, U>(&mut self, key: &Q, mut update_fn: U) -> Option<&V>
    where
        U: FnMut(&mut V),
        K: Borrow<Q> + Hash + Eq,
        Q: ToOwned<Owned = K> + Hash + Eq + ?Sized,
    {
        let v = self.inner.0.items.get_mut(key)?;
        let old_key = (self.field)(v);
        update_fn(v);
        self.inner
            .0
            .store
            .update(old_key, key.to_owned(), (self.field)(v));
        Some(v)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Hash + Eq,
        Q: ToOwned<Owned = K> + Hash + Eq + ?Sized,
    {
        let removed = self.inner.0.items.remove(key)?;
        self.inner
            .0
            .store
            .delete((self.field)(&removed), &key.to_owned());
        Some(removed)
    }
}

impl<S, K, V, F> Deref for LkupHashMap<S, K, V, F> {
    type Target = ro::LkupBaseMap<S, HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::{MultiPosIndex, UniquePosHash};

    #[derive(Debug, PartialEq)]
    struct Car(usize, String);

    #[test]
    fn map_key_string() {
        let mut m = LkupHashMap::<MultiPosIndex<_, String>, _, Car, _>::new(|c| c.0);
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
            m.update("BMW", |c| c.0 = 1_000)
        );
        assert!(m.lkup().contains_key(1_000));

        assert_eq!(None, m.update("NotFound", |_c| {}));

        // remove
        assert_eq!(2, m.len());
        assert_eq!(Some(Car(1_000, String::from("BMW"))), m.remove("BMW"));
        assert!(!m.contains_key("BMW"));
        assert!(!m.lkup().contains_key(1_000));
        assert_eq!(1, m.len());
    }

    #[test]
    fn map_key_usize() {
        let mut m = LkupHashMap::<UniquePosHash<String, usize>, _, Car, _>::new(|c| c.1.clone());
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
            m.update(&1, |c| c.1 = String::from("VW"))
        );
        assert!(m.lkup().contains_key("VW"));

        assert_eq!(None, m.update(&1_000, |_c| {}));

        // remove
        assert_eq!(2, m.len());
        assert_eq!(Some(Car(1, String::from("VW"))), m.remove(&1));
        assert!(!m.contains_key(&1));
        assert!(!m.lkup().contains_key("VW"));
        assert_eq!(1, m.len());
    }
}
