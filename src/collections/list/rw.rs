//! `Read write` implementations for lookup collections `List` like `Vec`
//!
use std::ops::Deref;

use crate::{collections::list::ro, lookup::store::Store};

///
/// `List` is a list with one `Store`.
/// This means, one `Lookup`.
///
#[derive(Debug)]
pub struct LVec<S, I, F> {
    field: F,
    inner: ro::LVec<S, I>,
}

impl<S, I, F> LVec<S, I, F>
where
    S: Store<Pos = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new<V>(field: F, items: V) -> Self
    where
        F: Clone,
        V: Into<Vec<I>>,
    {
        Self {
            field: field.clone(),
            inner: ro::LVec::new(field, items),
        }
    }

    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) -> usize {
        push(&mut self.inner.items, item, |i, idx| {
            self.inner.store.insert((self.field)(i), idx);
        })
    }

    /// Update an existing `Item` on given `Position` from the List.
    /// If the `Position` exist, the method returns an `Some` with reference to the updated Item.
    /// If not, the method returns `None`.
    pub fn update<U>(&mut self, pos: usize, mut update_fn: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        update(
            self.inner.items.as_mut_slice(),
            pos,
            &self.field,
            &mut update_fn,
            |old_key, pos, new_key| {
                self.inner.store.update(old_key, pos, new_key);
            },
        )
    }
}

impl<S, I, F> Deref for LVec<S, I, F> {
    type Target = ro::LVec<S, I>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[inline]
fn push<I, Trigger>(items: &mut Vec<I>, item: I, mut trigger: Trigger) -> usize
where
    Trigger: FnMut(&I, usize),
{
    let idx = items.len();
    trigger(&item, idx);
    items.push(item);
    idx
}

#[inline]
fn update<'a, I, F, K, U, Trigger>(
    items: &'a mut [I],
    pos: usize,
    field: &F,
    mut update: U,
    mut trigger: Trigger,
) -> Option<&'a I>
where
    F: Fn(&I) -> K,
    U: FnMut(&mut I),
    Trigger: FnMut(K, usize, K),
{
    items.get_mut(pos).map(|item| {
        let old_key = field(item);
        update(item);
        trigger(old_key, pos, field(item));
        &*item
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::MultiPosHash;

    #[derive(PartialEq, Debug, Clone)]
    struct Person {
        id: usize,
        name: String,
    }

    impl Person {
        fn new(id: usize, name: &str) -> Self {
            Self {
                id,
                name: name.into(),
            }
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[test]
    fn lvec() {
        let mut v = LVec::<MultiPosHash, _, _>::new(Person::name, [Person::new(0, "Paul")]);
        v.push(Person::new(1, "Anna"));

        assert!(!v.is_empty());
        assert_eq!(2, v.len());

        assert_eq!(
            &Person::new(1, "Anna"),
            v.lkup().get_by_key("Anna").next().unwrap()
        );

        // id 101 not exist
        assert_eq!(None, v.update(101, |p| { p.id = 102 }));
        assert_eq!(
            Some(&Person::new(99, "Anna")),
            v.update(1, |p| { p.id = 99 })
        );
        assert_eq!(
            &Person::new(99, "Anna"),
            v.lkup().get_by_key("Anna").next().unwrap()
        );

        let view = v.create_lkup_view([String::from("Paul")]);
        assert_eq!(
            &Person::new(0, "Paul"),
            view.get_by_key("Paul").next().unwrap()
        );
        assert!(view.get_by_key("Anna").next().is_none());
    }
}
