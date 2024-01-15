//! A lookup has a `Key` (for what you are looking for) and
//! get the `Position`s (Index in a Vec for example) where the `Key` is stored.
//!

/// `KeyPosition` has two expressions:
/// - `unique`: for a given `Key` exist none or one `Position`
/// - `multi`:  for a given `Key` exist none or many `Position`
///
pub trait KeyPosition<P> {
    /// Create a new `KeyPosition` with the initial value `pos`.
    fn new(pos: P) -> Self;

    /// Add a new `pos`.
    fn add_pos(&mut self, pos: P);
    /// Remove a `pos`. If the return value is `true`, than the last position was removed.
    fn remove_pos(&mut self, pos: &P) -> bool;
    /// Returns all saved `pos` as slice.
    fn as_slice(&self) -> &[P];
}

/// `UniqueKeyPositon` is an optional container for none or maximal one `Key` position.
///
/// ## Panics
/// Panics, the Posion must be unique, so you can not add a further `pos` ([UniqueKeyPositon::add_pos]) .
///
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct UniqueKeyPositon<P = usize>(Option<[P; 1]>);

impl<P> KeyPosition<P> for UniqueKeyPositon<P>
where
    P: PartialEq,
{
    /// Create a new Position.
    fn new(pos: P) -> Self {
        Self(Some([pos]))
    }

    /// ## Panics
    /// Panics, the Posion must be unique, so you can not add a further `pos`.
    fn add_pos(&mut self, _pos: P) {
        // maybe reuse this key, if the value is None, so can add_pos set a new Some value
        panic!("unique UniqueKeyPositon can not add a new position")
    }

    /// If it is Some, than remove the `pos` and set the value to Nome.
    /// If it is already None, than ignore the `pos`.
    fn remove_pos(&mut self, pos: &P) -> bool {
        match self.0.as_ref() {
            Some(inner) if &inner[0] == pos => {
                self.0 = None;
                true
            }
            Some(_) => false,
            None => true,
        }
    }

    /// Returns empty slice, if the value is None, otherwise return a slice with the `pos`.
    fn as_slice(&self) -> &[P] {
        self.0
            .as_ref()
            .map_or_else(|| [].as_slice(), |idx| idx.as_slice())
    }
}

/// `MultiKeyPositon` is an container for empty or many `Key` positions.
///
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct MultiKeyPositon<P = usize>(Vec<P>);

impl<P> KeyPosition<P> for MultiKeyPositon<P>
where
    P: Ord + PartialEq,
{
    /// Create a new Position collection with the initial pos.
    fn new(pos: P) -> Self {
        Self(vec![pos])
    }

    /// Add new Positin to a sorted collection.
    /// Duplicate Positions are ignored.
    fn add_pos(&mut self, pos: P) {
        if let Err(idx) = self.0.binary_search(&pos) {
            self.0.insert(idx, pos);
        }
    }

    /// Remove one Position and return left free Indices.
    fn remove_pos(&mut self, pos: &P) -> bool {
        self.0.retain(|v| v != pos);
        self.0.is_empty()
    }

    /// Returns all saved `pos` as slice.
    fn as_slice(&self) -> &[P] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {

    mod unique_key_position {
        use super::super::*;

        impl<P> From<[P; 1]> for UniqueKeyPositon<P> {
            fn from(index: [P; 1]) -> Self {
                Self(Some(index))
            }
        }

        #[test]
        fn unique_new() {
            assert_eq!(UniqueKeyPositon::new(7), [7; 1].into());
            assert_eq!(UniqueKeyPositon::new(7).as_slice(), &[7]);
        }

        #[test]
        #[should_panic]
        fn add_pos_with_panic() {
            UniqueKeyPositon::new(1).add_pos(2);
        }

        #[test]
        fn as_position() {
            let mut x = UniqueKeyPositon::new(1);

            assert_eq!(x.as_slice(), &[1; 1]);

            assert!(x.remove_pos(&1));
            assert_eq!(x.as_slice(), &[]);
        }

        #[test]
        fn remove_pos() {
            let mut x = UniqueKeyPositon::new(1);

            // invalid key
            assert!(!x.remove_pos(&2));

            // remove twice
            assert!(x.remove_pos(&1));
            assert!(x.remove_pos(&1));

            // the key is alway removed, also returns true for ever
            assert!(x.remove_pos(&2));
        }
    }

    mod multi_key_indices {
        use super::super::*;

        #[test]
        fn multi_new() {
            assert_eq!(MultiKeyPositon::new(7), MultiKeyPositon(vec![7]));
            assert_eq!(MultiKeyPositon::new(7).as_slice(), &[7]);
        }

        #[test]
        fn multi_position_and_are_ordered() {
            let mut m = MultiKeyPositon::new(2);
            assert_eq!(&[2], m.as_slice());

            m.add_pos(1);
            assert_eq!(&[1, 2], m.as_slice());

            m.add_pos(0);
            assert_eq!(&[0, 1, 2], m.as_slice());
        }

        #[test]
        fn multi_duplicate() {
            let mut m = MultiKeyPositon::new(1);
            assert_eq!(&[1], m.as_slice());

            // ignore add: 1, 1 exists already
            m.add_pos(1);
            assert_eq!(&[1], m.as_slice());
        }

        #[test]
        fn multi_ordered() {
            let mut m = MultiKeyPositon::new(5);
            assert_eq!(&[5], m.as_slice());

            m.add_pos(3);
            m.add_pos(1);
            m.add_pos(4);

            assert_eq!(&[1, 3, 4, 5], m.as_slice());
        }

        #[test]
        fn remove() {
            let mut m = MultiKeyPositon::new(5);
            m.add_pos(3);
            m.add_pos(2);

            assert_eq!(&[2, 3, 5], m.as_slice());

            assert!(!m.remove_pos(&3));

            // remove 3 twice, no problem
            assert!(!m.remove_pos(&3));

            assert!(!m.remove_pos(&2));

            // remove last pos
            assert!(m.remove_pos(&5));

            // remove after last
            assert!(m.remove_pos(&3));
        }
    }
}
