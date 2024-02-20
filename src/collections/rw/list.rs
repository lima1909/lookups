use crate::{
    collections::Retriever,
    lookup::store::{Store, View, ViewCreator},
};

///
/// `List` is a list with one `Store`.
/// This means, one `Index`.
///
#[derive(Debug)]
pub struct LVec<S, I, F> {
    store: S,
    items: Vec<I>,
    field: F,
}

impl<S, I, F> LVec<S, I, F>
where
    S: Store<Pos = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        let mut s = Self {
            field,
            store: S::with_capacity(iter.len()),
            items: Vec::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|item| {
            s.push(item);
        });

        s
    }

    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) -> usize {
        push(&mut self.items, item, |i, idx| {
            self.store.insert((self.field)(i), idx);
        })
    }

    /// Update an existing `Item` on given `Position` from the List.
    /// If the `Position` exist, the method returns an `Some` with reference to the updated Item.
    /// If not, the method returns `None`.
    pub fn update<U>(&mut self, pos: usize, mut update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.items.get_mut(pos).map(|item| {
            let key = (self.field)(item);
            update(item);
            self.store.update(key, pos, (self.field)(item));
            &*item
        })
    }

    pub fn lkup(&self) -> Retriever<'_, &S, Vec<I>> {
        Retriever::new(&self.store, &self.items)
    }

    pub fn create_lkup_view<'a, It, Q>(
        &'a self,
        keys: It,
    ) -> Retriever<'_, View<S::Lookup, Q>, Vec<I>>
    where
        S: ViewCreator<'a, Q>,
        It: IntoIterator<Item = <S as ViewCreator<'a, Q>>::Key>,
    {
        let view = self.store.create_view(keys);
        Retriever::new(view, &self.items)
    }
}

#[inline]
fn push<I, Trigger>(items: &mut Vec<I>, item: I, mut insert: Trigger) -> usize
where
    Trigger: FnMut(&I, usize),
{
    let idx = items.len();
    insert(&item, idx);
    items.push(item);
    idx
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
        let persons = vec![Person::new(0, "Paul")];

        let mut v = LVec::<MultiPosHash, _, _>::from_iter(Person::name, persons.into_iter());
        v.push(Person::new(1, "Anna"));

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

        let view = v.create_lkup_view::<_, &str>([String::from("Paul")]);
        assert_eq!(
            &Person::new(0, "Paul"),
            view.get_by_key("Paul").next().unwrap()
        );
        assert!(view.get_by_key("Anna").next().is_none());
    }
}
