pub mod ro;

use std::collections::{BTreeMap, HashMap};

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap as HHashMap;

macro_rules! itemer {
    (
        $( $itemer:ident ), + $(,)*
    ) => {
        $(

            impl<Q, K, T> crate::collections::Itemer<Q> for $itemer<K, T>
            where
                K: std::borrow::Borrow<Q> + std::hash::Hash + Eq + Ord,
                Q: std::hash::Hash + Eq + Ord,
            {
                type Output = T;

                fn item(&self, pos: &Q) -> &Self::Output {
                    &self[pos]
                }
            }
        )+
    };
}

itemer!(HashMap, BTreeMap);

#[cfg(feature = "hashbrown")]
itemer!(HHashMap);

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
