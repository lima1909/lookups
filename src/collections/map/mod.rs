//! `Map`s are collections like like `HashMap`, `BTreeMap`, ...
//!

pub mod ro;

use std::{hash::Hash, ops::Index};

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
