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

    pub fn idx<Q>(&self) -> Retriever<'_, S, &[I], Q>
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
    struct Car(usize, String);

    impl Car {
        fn id(&self) -> usize {
            self.0
        }

        fn name(&self) -> String {
            self.1.clone()
        }
    }

    #[test]
    fn lvec_usize() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LVec::<UniqueUIntLookup, _>::new(Car::id, items);

        let l = v.idx();
        assert!(l.contains_key(99));
        assert!(!l.contains_key(1_000));

        assert_eq!(
            vec![&Car(99, "Audi".into())],
            l.get_by_key(99).collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(0, "BMW".into()), &Car(99, "Audi".into())],
            l.get_by_many_keys([0, 99]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn lvec_string() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LVec::<UniqueMapLookup, _>::new(Car::name, items);

        let l = v.idx();
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
    }
}
