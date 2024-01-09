use crate::lookup::store::{Lookup, Store};
use std::ops::Deref;

/// Is a read only lookup extenstion for a [`std::vec::Vec`].
pub struct LVec<S, T> {
    store: S,
    items: Vec<T>,
}

impl<S, T> LVec<S, T>
where
    S: Store<Pos = usize>,
{
    pub fn new<F, K>(field: F, items: Vec<T>) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Pos = usize>,
    {
        let mut store = S::with_capacity(items.len());
        items.iter().enumerate().for_each(|(pos, item)| {
            store.insert(field(item), pos);
        });

        Self { store, items }
    }

    pub fn find<Q>(&self, key: Q) -> impl Iterator<Item = &T>
    where
        S: Lookup<Q, Pos = usize>,
    {
        let pos = self.store.pos_by_key(key);
        pos.iter().map(|idx| &self.items[*idx])
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
        let items = vec![Car(0, "BMW".into())];
        let v = LVec::<UniqueUIntLookup, _>::new(Car::id, items);

        assert_eq!(vec![&Car(0, "BMW".into())], v.find(0).collect::<Vec<_>>());
    }

    #[test]
    fn lvec_string() {
        let items = vec![Car(0, "BMW".into())];
        let v = LVec::<UniqueMapLookup, _>::new(Car::name, items);

        assert_eq!(
            vec![&Car(0, "BMW".into())],
            v.find("BMW").collect::<Vec<_>>()
        );
    }
}
