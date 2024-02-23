//! `List`s are collections like `Vec`, `Slice`, ...
//!

pub mod ro;
pub mod rw;

use std::ops::Index;

pub use rw::LkupVec;

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
