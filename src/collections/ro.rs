//! `Read only` implementations for lookup collections like:
//! - `LVec` a lookup extended vec
//! - `LHashMap` a lookup extended map
//!

use super::Retriever;
use crate::lookup::store::{Lookup, LookupExt, Store, ToStore};
use std::ops::Deref;

/// [`LVec`] is a read only lookup extenstion for a [`std::vec::Vec`].
///
/// # Example
///
/// ```
/// use lookups::{collections::ro::LVec, lookup::UniqueHashLookup};
///
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
/// let vec = LVec::<UniqueHashLookup, _>::new(|p| p.name.clone(), data);
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

    pub fn lkup<Q>(&self) -> Retriever<'_, S, Vec<I>, Q>
    where
        S: Lookup<Q, Pos = usize> + LookupExt,
    {
        Retriever::new(&self.store, &self.items)
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
/// use lookups::{collections::ro::LHashMap, lookup::UniqueIndexLookup};
///
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
/// let map = LHashMap::<UniqueIndexLookup<_, _>, _, _>::new(|p| p.id, data);
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

    pub fn lkup<Q>(&self) -> Retriever<'_, S, crate::HashMap<K, V>, Q>
    where
        S: Lookup<Q, Pos = K> + LookupExt,
    {
        Retriever::new(&self.store, &self.items)
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
    use crate::lookup::{MultiIndexLookup, UniqueHashLookup, UniqueIndexLookup};

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    impl Car {
        fn id(&self) -> u16 {
            self.0
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
        let m = LHashMap::<MultiIndexLookup<u16, String>, _, _>::new(Car::id, items);

        assert!(m.contains_key("BMW"));

        assert!(m.lkup().contains_key(1));
        assert!(!m.lkup().contains_key(1_000));
    }

    #[test]
    fn lvec_u16() {
        let items = vec![Car(99, "Audi".into()), Car(1, "BMW".into())];
        let v = LVec::<UniqueIndexLookup<u16, _>, _>::new(Car::id, items);

        let l = v.lkup();

        assert!(l.contains_key(1));
        assert!(l.contains_key(99));
        assert!(!l.contains_key(1_000));

        assert_eq!(
            vec![&Car(1, "BMW".into())],
            l.get_by_key(1).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Car(99, "Audi".into())],
            l.get_by_key(99).collect::<Vec<_>>()
        );
        assert!(l.get_by_key(98).next().is_none());

        assert_eq!(
            vec![&Car(1, "BMW".into()), &Car(99, "Audi".into())],
            l.get_by_many_keys([1, 99]).collect::<Vec<_>>()
        );

        assert_eq!(1, v.lkup().min_key().unwrap());
        assert_eq!(99, v.lkup().max_key().unwrap());

        assert_eq!(vec![1, 99], v.lkup().keys().collect::<Vec<_>>());
    }

    #[test]
    fn lvec_string() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LVec::<UniqueHashLookup, _>::new(Car::name, items);

        let l = v.lkup();
        assert!(l.contains_key("Audi"));
        assert!(!l.contains_key("VW"));

        assert_eq!(
            vec![&Car(0, "BMW".into())],
            l.get_by_key("BMW").collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(99, "Audi".into()), &Car(0, "BMW".into())],
            l.get_by_many_keys(["Audi", "BMW"]).collect::<Vec<_>>()
        );

        let keys = v
            .lkup::<&str>()
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        assert!(keys.contains("Audi"));
        assert!(keys.contains("BMW"));
    }
}
