//! `Map`s are collections like like `HashMap`, `BTreeMap`, ...
//!

pub mod ro;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap as HHashMap;

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::Index,
};

pub struct MapIndex<'a, I>(&'a I);

impl<'a, I, Q> Index<&'a Q> for MapIndex<'a, I>
where
    I: Index<&'a Q>,
    Q: Eq + Hash + Ord + ?Sized,
{
    type Output = I::Output;

    fn index(&self, index: &'a Q) -> &Self::Output {
        self.0.index(index)
    }
}

macro_rules! create_store {
    (
        $( $itemer:ident ), + $(,)*
    ) => {
        $(

            impl<S, K, T> crate::collections::StoreCreator<S> for $itemer<K, T>
            where
                S: crate::lookup::store::Store<Pos = K>,
                K: Clone,
            {
                type Item = T;

                fn create_store<F>(&self, field: &F) -> S
                where
                    F: Fn(&Self::Item) -> S::Key,
                {
                    let mut store = S::with_capacity(self.len());

                    self.iter().map(|(k, v)| (field(v), k.clone()))
                        .for_each(|(key, pos)| store.insert(key, pos));

                    store
                }
            }

        )+
    };
}

create_store!(HashMap, BTreeMap);

#[cfg(feature = "hashbrown")]
create_store!(HHashMap);
