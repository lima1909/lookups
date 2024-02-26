//! `Read only` implementations for lookup collections `LkupMap` like `HashMap`, `BTreeMap`
//!

use crate::collections::{map::MapIndex, Retriever};
use crate::lookup::store::{Store, View, ViewCreator};
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
/// use lookups::{collections::map::ro::LkupHashMap, lookup::index::UniquePosIndex};
///
/// let map = LkupHashMap::<UniquePosIndex<_, _>, _, _>::from_iter(|p| p.id, persons);
///
/// assert!(map.contains_key("Paul"));     // conventionally HashMap access with String - Key
/// assert!(map.lkup().contains_key(2)); // lookup with usize - Key
///
/// assert_eq!(
///     &Person{id: 5, name:  "Mario".into()},
///     // get a Person by an given Key
///     map.lkup().get_by_key(5).next().unwrap()
/// );
///
/// assert_eq!(
///     vec![&Person{id: 0, name:  "Paul".into()}, &Person{id: 2, name:  "Jasmin".into()}],
///     // get many Persons by given many Keys
///     map.lkup().get_by_many_keys([0, 2]).collect::<Vec<_>>(),
/// );
/// ```
///

#[cfg(feature = "hashbrown")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;

#[cfg(not(feature = "hashbrown"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug)]
#[repr(transparent)]
pub struct LkupHashMap<S, K, V>(pub(crate) LkupBaseMap<S, HashMap<K, V>>);

impl<S, K, V> LkupHashMap<S, K, V>
where
    S: Store<Pos = K>,
{
    pub fn new<F>(field: F, map: HashMap<K, V>) -> Self
    where
        F: Fn(&V) -> S::Key,
        K: Clone,
    {
        let mut store = S::with_capacity(map.len());
        map.iter()
            .map(|(k, v)| (field(v), k.clone()))
            .for_each(|(key, pos)| store.insert(key, pos));

        Self(LkupBaseMap { store, items: map })
    }

    pub fn from_iter<I, F>(field: F, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        F: Fn(&V) -> S::Key,
        K: Hash + Eq + Clone,
    {
        Self::new(field, HashMap::from_iter(iter))
    }
}

impl<S, K, V> Deref for LkupHashMap<S, K, V> {
    type Target = LkupBaseMap<S, HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct LkupBaseMap<S, I> {
    pub(crate) store: S,
    pub(crate) items: I,
}

impl<S, I> LkupBaseMap<S, I>
where
    S: Store,
{
    pub fn lkup(&self) -> Retriever<&S, MapIndex<'_, I>> {
        Retriever::new(&self.store, MapIndex(&self.items))
    }

    pub fn create_lkup_view<'a, It>(
        &'a self,
        keys: It,
    ) -> Retriever<View<S::Lookup>, MapIndex<'_, I>>
    where
        S: ViewCreator<'a>,
        It: IntoIterator<Item = <S as ViewCreator<'a>>::Key>,
    {
        let view = self.store.create_view(keys);
        Retriever::new(view, MapIndex(&self.items))
    }
}

impl<S, I> Deref for LkupBaseMap<S, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::index::MultiPosIndex;

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    #[test]
    fn map_u16() {
        let mut items = HashMap::new();
        items.insert("Audi".into(), Car(99, "Audi".into()));
        items.insert("BMW".into(), Car(1, "BMW".into()));
        let m = LkupHashMap::<MultiPosIndex<u16, String>, _, _>::new(|c| c.0, items);

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));
    }
}
