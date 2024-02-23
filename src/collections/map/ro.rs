//! `Read only` implementations for lookup collections `LkupMap` like `HashMap`, `BTreeMap`
//!

use crate::collections::{map::MapIndex, Retriever, StoreCreator};
use crate::lookup::store::{Store, View, ViewCreator};
use std::ops::Deref;

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
/// let mut persons = std::collections::HashMap::new();
/// persons.insert(String::from("Paul")  , Person{id: 0, name: "Paul".into()});
/// persons.insert(String::from("Mario") , Person{id: 5, name: "Mario".into()});
/// persons.insert(String::from("Jasmin"), Person{id: 2, name: "Jasmin".into()});
///
/// use lookups::{collections::map::ro::LkupMap, lookup::UniquePosIndex};
///
/// let map = LkupMap::<UniquePosIndex<_, _>, _>::new(|p| p.id, persons);
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
pub struct LkupMap<S, I> {
    store: S,
    items: I,
}

impl<S, I> LkupMap<S, I> {
    pub fn new<F>(field: F, items: I) -> Self
    where
        S: Store,
        I: StoreCreator<S>,
        F: Fn(&I::Item) -> S::Key,
    {
        Self {
            store: items.create_store(&field),
            items,
        }
    }

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

impl<S, I> Deref for LkupMap<S, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::MultiPosIndex;

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    #[test]
    fn map_u16() {
        let mut items = std::collections::HashMap::new();
        items.insert("Audi".into(), Car(99, "Audi".into()));
        items.insert("BMW".into(), Car(1, "BMW".into()));
        let m = LkupMap::<MultiPosIndex<u16, String>, _>::new(|c| c.0, items);

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));
    }
}