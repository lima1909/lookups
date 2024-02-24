//! `Read write` implementations for lookup collections `Vec`.
//!

use crate::{collections::list::ro, lookup::store::Store};
use std::ops::Deref;

/// [`LkupVec`] is a [`std::vec::Vec`] with one `Lookup`.
///
/// # Example
///
/// ```
/// #[derive(PartialEq, Debug)]
/// struct Person {
///     id: usize,
///     name: String,
/// }
///
/// use lookups::{collections::LkupVec, lookup::MultiPosHash};
///
/// let mut vec = LkupVec::<MultiPosHash, Person, _>::new(|p| p.name.clone());
///
/// vec.push(Person{id: 0, name: "Paul".into()});
/// vec.push(Person{id: 5, name: "Mario".into()});
/// vec.push(Person{id: 2, name: "Jasmin".into()});
///
/// assert!(vec.lkup().contains_key("Paul")); // lookup with a given Key
///
/// assert_eq!(
///     &Person{id: 5, name:  "Mario".into()},
///     // get a Person by an given Key: "Mario"
///     vec.lkup().get_by_key("Mario").next().unwrap()
/// );
///
/// assert_eq!(
///     vec![&Person{id: 0, name:  "Paul".into()}, &Person{id: 2, name:  "Jasmin".into()}],
///     // get many a Person by an many given Key
///     vec.lkup().get_by_many_keys(["Paul", "Jasmin"]).collect::<Vec<_>>(),
/// );
/// ```
///

#[derive(Debug)]
pub struct LkupVec<S, I, F> {
    field: F,
    inner: ro::LkupList<S, Vec<I>>,
}

impl<S, I, F> LkupVec<S, I, F>
where
    S: Store<Pos = usize>,
    F: Fn(&I) -> S::Key,
{
    // pub fn new<V>(field: F, items: V) -> Self
    // where
    //     V: Into<Vec<I>>,
    //     F: Clone,
    // {
    //     Self {
    //         inner: ro::LkupList::new(field.clone(), items.into()),
    //         field,
    //     }
    // }

    pub fn new(field: F) -> Self
    where
        F: Clone,
    {
        Self {
            inner: ro::LkupList::new(field.clone(), Vec::new()),
            field,
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

impl<S, I, F> Deref for LkupVec<S, I, F> {
    type Target = ro::LkupList<S, Vec<I>>;

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
    fn lkupvec() {
        let mut v = LkupVec::<MultiPosHash, _, _>::new(Person::name);
        v.push(Person::new(1, "Anna"));
        v.push(Person::new(0, "Paul"));

        assert!(!v.is_empty());
        assert_eq!(2, v.len());

        assert_eq!(
            &Person::new(1, "Anna"),
            v.lkup().get_by_key("Anna").next().unwrap()
        );

        // id 101 not exist
        // assert_eq!(None, v.update(101, |p| { p.id = 102 }));
        assert_eq!(
            Some(&Person::new(99, "Anna")),
            v.update(0, |p| { p.id = 99 })
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
