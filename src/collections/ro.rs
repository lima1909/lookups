use super::Retriever;
use crate::lookup::store::{Lookup, Store};
use std::ops::Deref;

/// `LVec` is a read only lookup extenstion for a [`std::vec::Vec`].
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
        use crate::lookup::store::ToStore;

        let v = items.into();

        Self {
            store: v.iter().enumerate().to_store(field),
            items: v,
        }
    }

    pub fn lookup<Q>(&self) -> Retriever<'_, S, Vec<I>, Q>
    where
        S: Lookup<Q, Pos = usize>,
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

#[cfg(test)]
mod tests {
    use crate::lookup::{UniqueMapLookup, UniqueUIntLookup};

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
    fn lvec_u16() {
        let items = vec![Car(99, "Audi".into()), Car(1, "BMW".into())];
        let v = LVec::<UniqueUIntLookup<u16, _>, _>::new(Car::id, items);

        let l = v.lookup();

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

        assert_eq!(1, v.lookup().min_key().unwrap());
        assert_eq!(99, v.lookup().max_key().unwrap());

        assert_eq!(vec![1, 99], v.lookup().keys().collect::<Vec<_>>());
    }

    #[test]
    fn lvec_string() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LVec::<UniqueMapLookup, _>::new(Car::name, items);

        let l = v.lookup();
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
            .lookup::<&str>()
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        assert!(keys.contains("Audi"));
        assert!(keys.contains("BMW"));
    }
}
