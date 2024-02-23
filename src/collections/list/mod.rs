//! `List`s are collections like `Vec`, `Slice`, ...
//!

pub mod ro;
pub mod rw;

use crate::{collections::StoreCreator, lookup::store::Store};
use std::ops::Index;

pub struct ListIndex<'a, I>(&'a I);

impl<I> Index<&usize> for ListIndex<'_, I>
where
    I: Index<usize>,
{
    type Output = I::Output;

    fn index(&self, index: &usize) -> &Self::Output {
        self.0.index(*index)
    }
}

impl<S, T> StoreCreator<S> for Vec<T>
where
    S: Store<Pos = usize>,
{
    type Item = T;

    fn create_store<F>(&self, field: &F) -> S
    where
        F: Fn(&Self::Item) -> S::Key,
    {
        self.iter().create_store(field)
    }
}

impl<S, T, const N: usize> StoreCreator<S> for [T; N]
where
    S: Store<Pos = usize>,
{
    type Item = T;

    fn create_store<F>(&self, field: &F) -> S
    where
        F: Fn(&Self::Item) -> S::Key,
    {
        self.iter().create_store(field)
    }
}

impl<S, T> StoreCreator<S> for &[T]
where
    S: Store<Pos = usize>,
{
    type Item = T;

    fn create_store<F>(&self, field: &F) -> S
    where
        F: Fn(&Self::Item) -> S::Key,
    {
        self.iter().create_store(field)
    }
}

impl<'a, S, T> StoreCreator<S> for std::slice::Iter<'a, T>
where
    S: Store<Pos = usize>,
{
    type Item = T;

    fn create_store<F>(&self, field: &F) -> S
    where
        F: Fn(&Self::Item) -> S::Key,
    {
        let mut store = S::with_capacity(self.len());

        self.clone()
            .enumerate()
            .for_each(|(pos, item)| store.insert(field(item), pos));

        store
    }
}
