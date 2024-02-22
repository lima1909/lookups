pub mod ro;
pub mod rw;

use crate::collections::Itemer;
// use std::ops::Index;

impl<T> Itemer<usize> for Vec<T> {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
    }
}

impl<T, const N: usize> Itemer<usize> for [T; N] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
    }
}

impl<T> Itemer<usize> for &[T] {
    type Output = T;

    fn item(&self, pos: &usize) -> &Self::Output {
        &self[*pos]
    }
}

// pub struct ListItemer<'a, I: Index<usize>>(&'a I);

// impl<I> Itemer<usize> for ListItemer<'_, I>
// where
//     I: Index<usize>,
// {
//     type Output = I::Output;

//     fn item(&self, pos: &usize) -> &Self::Output {
//         self.0.index(*pos)
//     }
// }
