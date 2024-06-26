//! `Read only` implementations for lookup collections `LkupMap` like `HashMap`, `BTreeMap`
//!

use crate::collections::{map::MapIndex, View};
use crate::lookup::store::{position::KeyPosition, Lookup, Retriever, Store, ViewCreator};
use std::{hash::Hash, ops::Deref};

/// [`LkupMap`] is a read only `HashMap` which is extended by a given `Lookup` implementation.
///
/// # Example
///
/// ```
/// #[derive(PartialEq, Debug)]
/// struct Person {
///     id: usize,
///     name: String,
/// }
///
/// let persons = [
///     (String::from("Paul")  , Person{id: 0, name: "Paul".into()}),
///     (String::from("Mario") , Person{id: 5, name: "Mario".into()}),
///     (String::from("Jasmin"), Person{id: 2, name: "Jasmin".into()})
/// ];
///
/// use lookups::{collections::map::ro::LkupHashMap, IndexLookup, Lookup};
///
/// let map = LkupHashMap::from_iter(IndexLookup::with_unique_key(), |p| p.id, persons);
///
/// assert!(map.contains_key("Paul"));     // conventionally HashMap access with String - Key
/// assert!(map.contains_lkup_key(2)); // lookup with usize - Key
///
/// assert_eq!(
///     &Person{id: 5, name:  "Mario".into()},
///     // get a Person by an given Key
///     map.get_by_lkup_key(5).next().unwrap()
/// );
///
/// assert_eq!(
///     vec![&Person{id: 0, name:  "Paul".into()}, &Person{id: 2, name:  "Jasmin".into()}],
///     // get many Persons by given many Keys
///     map.get_by_many_lkup_keys([0, 2]).collect::<Vec<_>>(),
/// );
/// ```
///

#[cfg(feature = "hashbrown")]
pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug, Clone)]
pub struct LkupHashMap<S, K, V> {
    pub(crate) store: S,
    pub(crate) items: HashMap<K, V>,
}

impl<S, K, V> LkupHashMap<S, K, V>
where
    S: Store<Pos = K>,
{
    pub fn new<L, P, F>(lookup: L, field: F, items: HashMap<K, V>) -> Self
    where
        L: Lookup<S, P>,
        P: KeyPosition<Pos = K>,
        F: Fn(&V) -> S::Key,
        K: Clone,
    {
        let store = lookup.new_map_store(&field, items.iter());
        Self { store, items }
    }

    pub fn from_iter<L, P, F, I>(lookup: L, field: F, iter: I) -> Self
    where
        L: Lookup<S, P>,
        P: KeyPosition<Pos = K>,
        F: Fn(&V) -> S::Key,
        I: IntoIterator<Item = (K, V)>,
        K: Hash + Eq + Clone,
    {
        Self::new(lookup, field, HashMap::from_iter(iter))
    }

    pub fn contains_lkup_key<Q>(&self, key: Q) -> bool
    where
        S: Retriever<Q>,
        K: Hash + Eq,
    {
        self.store.key_exist(key)
    }

    pub fn get_by_lkup_key<Q>(&self, key: Q) -> impl Iterator<Item = &V>
    where
        S: Retriever<Q, Pos = K>,
        K: Hash + Eq,
    {
        self.store.pos_by_key(key).iter().map(|p| &self.items[p])
    }

    pub fn get_by_many_lkup_keys<It, Q>(&self, keys: It) -> impl Iterator<Item = &V>
    where
        S: Retriever<Q, Pos = K>,
        K: Hash + Eq,
        It: IntoIterator<Item = Q>,
    {
        self.store.pos_by_many_keys(keys).map(|p| &self.items[p])
    }

    pub fn lkup_ext(&self) -> &S::Target
    where
        S: Deref,
    {
        self.store.deref()
    }

    pub fn create_lkup_view<'a, It>(
        &'a self,
        keys: It,
    ) -> View<S::Retriever, MapIndex<'_, HashMap<K, V>>>
    where
        S: ViewCreator<'a>,
        It: IntoIterator<Item = <S as ViewCreator<'a>>::Key>,
    {
        let view = self.store.create_view(keys);
        View::new(view, MapIndex(&self.items))
    }
}

impl<S, K, V> Deref for LkupHashMap<S, K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IndexLookup;

    #[derive(Debug, PartialEq, Clone)]
    struct Car(u16, String);

    #[test]
    fn map_u16_and_clone() {
        let mut items = HashMap::new();
        items.insert(String::from("Audi"), Car(99, "Audi".into()));
        items.insert("BMW".into(), Car(1, "BMW".into()));
        let m = LkupHashMap::new(IndexLookup::with_multi_keys(), |c: &Car| c.0, items);

        assert!(m.contains_key("BMW"));

        assert!(m.contains_lkup_key(1));
        assert!(!m.contains_lkup_key(1_000));

        m.lkup_ext()
            .keys()
            .for_each(|key| assert!(m.contains_lkup_key(key)));

        assert_eq!(&Car(99, "Audi".into()), &m["Audi"]);

        let m2 = m.clone();
        assert!(m2.contains_lkup_key(1));
        assert_eq!(&Car(1, "BMW".into()), &m2["BMW"]);
        assert_eq!(
            vec![&Car(99, "Audi".into()), &Car(1, "BMW".into())],
            m2.get_by_many_lkup_keys([99, 1]).collect::<Vec<_>>()
        );
    }
}
