//! Here are defined parts to get an `Item` from a `Collection` based on a given `Position` (Index).

use std::{borrow::Borrow, hash::Hash, marker::PhantomData};

/// [`Itemer`] returns an Item or an Item-Iterator
/// which are stored at a given `Position` (Index)
/// in a collection (Vec, Array, Map ...).
///
pub trait Itemer<Pos> {
    type Output;

    /// Get the Item based on the given Position.
    ///
    /// #Panic
    ///
    /// If no Item exist for the given Position.
    fn item(&self, pos: &Pos) -> &Self::Output;

    /// Return an `Iterator` with all `Items`
    /// for a given `Slice` with `Position`s.
    fn items<'a, It>(&'a self, positions: It) -> Items<'a, Pos, Self, It>
    where
        It: Iterator<Item = &'a Pos>,
        Pos: 'a,
    {
        Items {
            items: self,
            positions,
            _pos: PhantomData,
        }
    }
}

/// `Itmes`is an `Iterator` which is created by the `Itemer` trait.
/// `Items` contains all `Items` for a given amount of `Position`s.
pub struct Items<'a, Pos, I: ?Sized, It> {
    items: &'a I,
    positions: It,
    _pos: PhantomData<Pos>,
}

impl<'a, Pos, I, It> Iterator for Items<'a, Pos, I, It>
where
    I: Itemer<Pos>,
    It: Iterator<Item = &'a Pos>,
    Pos: 'a,
{
    type Item = &'a I::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.positions.next().map(|pos| self.items.item(pos))
    }
}

//
// --------- Itemer Implementation --------------------------
//
impl<T> Itemer<usize> for &[T] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
    }
}

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
