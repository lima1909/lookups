use crate::{collections::map::ro, lookup::store::Store};
use std::{hash::Hash, ops::Deref};

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

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Hash + Eq + Clone,
    {
        self.inner.0.store.insert((self.field)(&v), k.clone());
        self.inner.0.items.insert(k, v)
    }

    pub fn update<U>(&mut self, k: K, mut update_fn: U) -> Option<&V>
    where
        U: FnMut(&mut V),
        K: Hash + Eq,
    {
        let v = self.inner.0.items.get_mut(&k)?;
        let old_key = (self.field)(v);
        update_fn(v);
        self.inner.0.store.update(old_key, k, (self.field)(v));
        Some(v)
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
    use crate::lookup::MultiPosIndex;

    #[derive(Debug, PartialEq)]
    struct Car(usize, String);

    #[test]
    fn map_rw() {
        let mut m = LkupHashMap::<MultiPosIndex<_, String>, _, Car, _>::new(|c| c.0);
        m.insert("Audi".into(), Car(99, "Audi".into()));
        m.insert("BMW".into(), Car(1, "BMW".into()));

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));

        // update
        assert_eq!(
            Some(&Car(1_000, "BMW".into())),
            m.update("BMW".into(), |c| c.0 = 1_000)
        );
        assert!(m.lkup().contains_key(1_000));

        assert_eq!(None, m.update("NotFound".into(), |_c| {}));
    }
}
