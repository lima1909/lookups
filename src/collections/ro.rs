//! `Read only` implementations for lookup collections like:
//! - `LVec` a lookup extended vec
//! - `LHashMap` a lookup extended map
//!

use super::Retriever;
use crate::lookup::store::{Store, ToStore, View, ViewCreator};
use std::ops::Deref;

/// [`LVec`] is a read only lookup extenstion for a [`std::vec::Vec`].
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
/// let data = [
///     Person{id: 0, name: "Paul".into()},
///     Person{id: 5, name: "Mario".into()},
///     Person{id: 2, name: "Jasmin".into()},
///     ];
///
/// use lookups::{collections::ro::LVec, lookup::UniquePosHash};
///
/// let vec = LVec::<UniquePosHash, _>::new(|p| p.name.clone(), data);
///
/// assert!(vec.lkup().contains_key("Paul")); // lookup with a given Key
///
/// assert_eq!(
///     &Person{id: 5, name:  "Mario".into()},
///     // get a Person by an given Key: "Mario"
///     vec.lkup().get_by_key("Mario").next().unwrap()
/// );
///
/// assert_eq!(
///     vec![&Person{id: 0, name:  "Paul".into()}, &Person{id: 2, name:  "Jasmin".into()}],
///     // get many a Person by an many given Key
///     vec.lkup().get_by_many_keys(["Paul", "Jasmin"]).collect::<Vec<_>>(),
/// );
/// ```
///
pub struct LVec<S, I> {
    store: S,
    items: Vec<I>,
}

impl<S, I> LVec<S, I> {
    pub fn new<F, V>(field: F, items: V) -> Self
    where
        F: Fn(&I) -> S::Key,
        S: Store<Pos = usize>,
        V: Into<Vec<I>>,
    {
        let v = items.into();

        Self {
            store: v.iter().enumerate().to_store(field),
            items: v,
        }
    }

    pub fn lkup(&self) -> Retriever<'_, &S, Vec<I>> {
        Retriever::new(&self.store, &self.items)
    }

    pub fn create_lkup_view<'a, It, Q>(
        &'a self,
        keys: It,
    ) -> Retriever<'_, View<S::Lookup, Q>, Vec<I>>
    where
        S: ViewCreator<'a, Q>,
        It: IntoIterator<Item = S::Key>,
    {
        let view = self.store.create_view(keys);
        Retriever::new(view, &self.items)
    }
}

impl<S, T> Deref for LVec<S, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

/// [`LHashMap`] is a read only `HashMap` which is extended by a given `Lookup` implementation.
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
/// let data = [
///     (String::from("Paul")  , Person{id: 0, name: "Paul".into()}),
///     (String::from("Mario") , Person{id: 5, name: "Mario".into()}),
///     (String::from("Jasmin"), Person{id: 2, name: "Jasmin".into()}),
///     ];
///
/// use lookups::{collections::ro::LHashMap, lookup::UniquePosIndex};
///
/// let map = LHashMap::<UniquePosIndex<_>, _, _>::new(|p| p.id, data);
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
pub struct LHashMap<S, K, V> {
    store: S,
    items: crate::HashMap<K, V>,
}

impl<S, K, V> LHashMap<S, K, V> {
    pub fn new<F, M>(field: F, items: M) -> Self
    where
        F: Fn(&V) -> S::Key,
        S: Store<Pos = K>,
        M: Into<crate::HashMap<K, V>>,
        K: Clone,
    {
        let m = items.into();

        Self {
            store: m.iter().map(|(k, v)| (k.clone(), v)).to_store(field),
            items: m,
        }
    }

    pub fn lkup(&self) -> Retriever<'_, &S, crate::HashMap<K, V>> {
        Retriever::new(&self.store, &self.items)
    }

    pub fn create_lkup_view<'a, It, Q>(
        &'a self,
        keys: It,
    ) -> Retriever<'_, View<S::Lookup, Q>, crate::HashMap<K, V>>
    where
        S: ViewCreator<'a, Q>,
        It: IntoIterator<Item = S::Key>,
    {
        let view = self.store.create_view(keys);
        Retriever::new(view, &self.items)
    }
}

impl<S, K, V> Deref for LHashMap<S, K, V> {
    type Target = crate::HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::{MultiPosIndex, UniquePosHash, UniquePosIndex};

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    impl Car {
        fn id(&self) -> usize {
            self.0.into()
        }

        fn name(&self) -> String {
            self.1.clone()
        }
    }

    #[test]
    fn map_u16() {
        let items = crate::HashMap::from([
            ("Audi".into(), Car(99, "Audi".into())),
            ("BMW".into(), Car(1, "BMW".into())),
        ]);
        let m = LHashMap::<MultiPosIndex<String>, _, _>::new(Car::id, items);

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));

        m.lkup()
            .keys()
            .for_each(|key| assert!(m.lkup().contains_key(key)));
    }

    #[test]
    fn lvec_u16() {
        let items = vec![Car(99, "Audi".into()), Car(1, "BMW".into())];
        let v = LVec::<UniquePosIndex<_>, _>::new(Car::id, items);

        assert!(v.lkup().contains_key(1));
        assert!(v.lkup().contains_key(99));
        assert!(!v.lkup().contains_key(1_000));

        assert_eq!(
            vec![&Car(1, "BMW".into())],
            v.lkup().get_by_key(1).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Car(99, "Audi".into())],
            v.lkup().get_by_key(99).collect::<Vec<_>>()
        );
        assert!(v.lkup().get_by_key(98).next().is_none());

        assert_eq!(
            vec![&Car(1, "BMW".into()), &Car(99, "Audi".into())],
            v.lkup().get_by_many_keys([1, 99]).collect::<Vec<_>>()
        );

        assert_eq!(1, v.lkup().min_key().unwrap());
        assert_eq!(99, v.lkup().max_key().unwrap());

        assert_eq!(vec![1, 99], v.lkup().keys().collect::<Vec<_>>());
    }

    #[test]
    fn lvec_string() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LVec::<UniquePosHash, _>::new(Car::name, items);

        assert!(v.lkup().contains_key("Audi"));
        assert!(!v.lkup().contains_key("VW"));

        assert_eq!(
            vec![&Car(0, "BMW".into())],
            v.lkup().get_by_key("BMW").collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(99, "Audi".into()), &Car(0, "BMW".into())],
            v.lkup()
                .get_by_many_keys(["Audi", "BMW"])
                .collect::<Vec<_>>()
        );

        let keys = v
            .lkup()
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        assert!(keys.contains("Audi"));
        assert!(keys.contains("BMW"));

        let view = v.create_lkup_view(["Audi".into()]);

        assert!(view.contains_key("Audi"));
        assert!(!view.contains_key("BMW"));

        assert_eq!(
            vec![&Car(99, "Audi".into())],
            view.get_by_key("Audi").collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(99, "Audi".into())],
            view.get_by_many_keys(["Audi", "BMW", "VW"])
                .collect::<Vec<_>>()
        );

        assert_eq!(vec![&String::from("Audi")], view.keys().collect::<Vec<_>>());
    }
}
