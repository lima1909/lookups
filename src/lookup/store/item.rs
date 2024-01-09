//! Here are defined parts to get an `Item` from a `Collection` based on a given `Position` (Index).

use std::{borrow::Borrow, hash::Hash};

/// [`ItemAt`] returns an Item which are stored at a given `Position` (Index)
/// in a collection (Vec, Array, Map ...) It is a replacement of [`std::ops::Index`].
///
pub trait ItemAt<Pos> {
    type Output;

    /// Get the Item based on the given Position.
    ///
    /// #Panic
    ///
    /// If no Item exist for the given Position.
    fn item(&self, pos: &Pos) -> &Self::Output;
}

impl<T> ItemAt<usize> for &[T] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
    }
}

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

impl<Q, K, T> ItemAt<Q> for HashMap<K, T>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq,
{
    type Output = T;

    fn item(&self, pos: &Q) -> &Self::Output {
        &self[pos]
    }
}
