pub mod ro;
pub mod rw;

use crate::{
    collections::{Itemer, StoreCreator},
    lookup::store::Store,
};

impl<T> Itemer<usize> for Vec<T> {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
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

impl<T, const N: usize> Itemer<usize> for [T; N] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
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

impl<T> Itemer<usize> for &[T] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
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
