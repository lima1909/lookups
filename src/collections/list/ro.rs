//! `Read only` implementations for lookup collections [`LkupList`] like `Vec`, `Slice`, ...
//!
use crate::collections::{list::ListIndex, Retriever};
use crate::lookup::store::{Store, View, ViewCreator};
use std::ops::{Deref, Index};

/// [`LkupList`] is a read only lookup extenstion for a [`std::vec::Vec`].
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
/// let data = [Person{id: 0, name: "Paul".into()},
///             Person{id: 5, name: "Mario".into()},
///             Person{id: 2, name: "Jasmin".into()}];
///
/// use lookups::{collections::list::ro::LkupList, lookup::hash::UniquePosHash};
///
/// let vec = LkupList::<UniquePosHash, _>::new(|p| p.name.clone(), data);
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
pub struct LkupList<S, I> {
    pub(crate) store: S,
    pub(crate) items: I,
}

impl<S, I> LkupList<S, I>
where
    S: Store<Pos = usize>,
    I: Index<usize>,
{
    pub fn new<F, T>(field: F, items: I) -> Self
    where
        I: AsRef<[T]>,
        F: Fn(&T) -> S::Key,
    {
        let items_ref = items.as_ref();
        let mut store = S::with_capacity(items_ref.len());
        items_ref
            .iter()
            .enumerate()
            .for_each(|(pos, item)| store.insert(field(item), pos));

        Self { store, items }
    }

    pub fn lkup(&self) -> Retriever<&S, ListIndex<'_, I>> {
        Retriever::new(&self.store, ListIndex(&self.items))
    }

    pub fn create_lkup_view<'a, It>(
        &'a self,
        keys: It,
    ) -> Retriever<View<S::Lookup>, ListIndex<'a, I>>
    where
        S: ViewCreator<'a>,
        It: IntoIterator<Item = <S as ViewCreator<'a>>::Key>,
    {
        let view = self.store.create_view(keys);
        Retriever::new(view, ListIndex(&self.items))
    }
}

impl<S, I: Index<usize>> Deref for LkupList<S, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::{hash::UniquePosHash, index::UniquePosIndex};

    #[derive(Debug, PartialEq)]
    struct Car(u16, String);

    impl Car {
        fn id(&self) -> usize {
            self.0.into()
        }

        fn name(&self) -> String {
            self.1.clone()
        }
    }

    #[test]
    fn lkuplist_u16() {
        let items = [Car(99, "Audi".into()), Car(1, "BMW".into())];
        let v = LkupList::<UniquePosIndex, _>::new(Car::id, items);

        assert_eq!(2, v.len());

        assert!(v.lkup().contains_key(1));
        assert!(v.lkup().contains_key(99));
        assert!(!v.lkup().contains_key(1_000));

        assert_eq!(
            vec![&Car(1, "BMW".into())],
            v.lkup().get_by_key(1).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Car(99, "Audi".into())],
            v.lkup().get_by_key(99).collect::<Vec<_>>()
        );
        assert!(v.lkup().get_by_key(98).next().is_none());

        assert_eq!(
            vec![&Car(1, "BMW".into()), &Car(99, "Audi".into())],
            v.lkup().get_by_many_keys([1, 99]).collect::<Vec<_>>()
        );

        assert_eq!(1, v.lkup().min_key().unwrap());
        assert_eq!(99, v.lkup().max_key().unwrap());

        assert_eq!(vec![1, 99], v.lkup().keys().collect::<Vec<_>>());
    }

    #[test]
    fn lkuplist_string() {
        let items = vec![Car(99, "Audi".into()), Car(0, "BMW".into())];
        let v = LkupList::<UniquePosHash, _>::new(Car::name, items);

        assert!(v.lkup().contains_key("Audi"));
        assert!(!v.lkup().contains_key("VW"));

        assert_eq!(
            vec![&Car(0, "BMW".into())],
            v.lkup().get_by_key("BMW").collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(99, "Audi".into()), &Car(0, "BMW".into())],
            v.lkup()
                .get_by_many_keys(["Audi", "BMW"])
                .collect::<Vec<_>>()
        );

        let keys = v
            .lkup()
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        assert!(keys.contains("Audi"));
        assert!(keys.contains("BMW"));

        let view = v.create_lkup_view(["Audi".into()]);

        assert!(view.contains_key("Audi"));
        assert!(!view.contains_key("BMW"));

        assert_eq!(
            vec![&Car(99, "Audi".into())],
            view.get_by_key("Audi").collect::<Vec<_>>()
        );

        assert_eq!(
            vec![&Car(99, "Audi".into())],
            view.get_by_many_keys(["Audi", "BMW", "VW"])
                .collect::<Vec<_>>()
        );

        assert_eq!(vec![&String::from("Audi")], view.keys().collect::<Vec<_>>());
    }
}
