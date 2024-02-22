pub mod ro;

use crate::collections::Itemer;
use std::{borrow::Borrow, hash::Hash};

#[cfg(feature = "hashbrown")]
impl<Q, K, T> Itemer<Q> for hashbrown::HashMap<K, T>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq,
{
    type Output = T;

    fn item(&self, pos: &Q) -> &Self::Output {
        &self[pos]
    }
}

impl<Q, K, T> Itemer<Q> for std::collections::HashMap<K, T>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq,
{
    type Output = T;

    fn item(&self, pos: &Q) -> &Self::Output {
        &self[pos]
    }
}

impl<Q, K, T> Itemer<Q> for std::collections::BTreeMap<K, T>
where
    K: Borrow<Q> + Hash + Eq + Ord,
    Q: Hash + Eq + Ord,
{
    type Output = T;

    fn item(&self, pos: &Q) -> &Self::Output {
        &self[pos]
    }
}
