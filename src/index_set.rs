use std::borrow::Borrow;
use std::cell::{Cell, UnsafeCell};
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Index;

use indexmap::IndexSet;
use stable_deref_trait::StableDeref;

/// Append-only version of `indexmap::IndexSet` where
/// insertion does not require mutable access
pub struct FrozenIndexSet<T> {
    set: UnsafeCell<IndexSet<T>>,
    /// Eq/Hash implementations can have side-effects, and using Rc it is possible
    /// for FrozenIndexSet::insert to be called on a key that itself contains the same
    /// `FrozenIndexSet`, whose `eq` implementation also calls FrozenIndexSet::insert
    ///
    /// We use this `in_use` flag to guard against any reentrancy.
    in_use: Cell<bool>,
}

// safety: UnsafeCell implies !Sync

impl<T: Eq + Hash> FrozenIndexSet<T> {
    pub fn new() -> Self {
        Self {
            set: UnsafeCell::new(Default::default()),
            in_use: Cell::new(false),
        }
    }
}

impl<T: Eq + Hash + StableDeref> FrozenIndexSet<T> {
    // these should never return &T
    // these should never delete any entries
    pub fn insert(&self, value: T) -> &T::Target {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            let (index, _was_vacant) = (*set).insert_full(value);
            &*(*set)[index]
        };
        self.in_use.set(false);
        ret
    }

    // these should never return &T
    // these should never delete any entries
    pub fn insert_full(&self, value: T) -> (usize, &T::Target) {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            let (index, _was_vacant) = (*set).insert_full(value);
            (index, &*(*set)[index])
        };
        self.in_use.set(false);
        ret
    }

    // TODO implement in case the standard Entry API gets improved
    // // TODO avoid double lookup
    // pub fn entry<Q: ?Sized>(&self, value: &Q) -> Entry<T, Q>
    //     where Q: Hash + Equivalent<T> + ToOwned<Owned = T>
    // {
    //     assert!(!self.in_use.get());
    //     self.in_use.set(true);
    //     unsafe {
    //         let set = self.set.get();
    //         match (*set).get_full(value) {
    //             Some((index, reference)) => {
    //                 Entry::Occupied(OccupiedEntry {
    //                     index,
    //                     reference,
    //                     set: &*set,
    //                 })
    //             }
    //             None => {
    //                 Entry::Vacant(VacantEntry {
    //                     value: Cow::Borrowed(value),
    //                     set: &*set,
    //                 })
    //             }
    //         }
    //     }
    // }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&T::Target>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            (*set).get(k).map(|x| &**x)
        };
        self.in_use.set(false);
        ret
    }

    pub fn get_full<Q: ?Sized>(&self, k: &Q) -> Option<(usize, &T::Target)>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            (*set).get_full(k).map(|(i, x)| (i, &**x))
        };
        self.in_use.set(false);
        ret
    }

    pub fn get_index(&self, index: usize) -> Option<&T::Target> {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            (*set).get_index(index).map(|r| &**r)
        };
        self.in_use.set(false);
        ret
    }

    /// Returns true if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use elsa::FrozenIndexSet;
    ///
    /// let set: FrozenIndexSet<_> = [1, 2, 3].iter().cloned().map(|n| Box::new(n)).collect();
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(2), false);
    /// ```
    pub fn contains<Q: ?Sized>(&self, t: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let map = self.set.get();
            (*map).contains(t)
        };
        self.in_use.set(false);
        ret
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        unsafe {
            let map = self.set.get();
            (*map).len()
        }
    }

    /// Returns true if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn into_set(self) -> IndexSet<T> {
        self.set.into_inner()
    }

    /// Get mutable access to the underlying [`IndexSet`].
    ///
    /// This is safe, as it requires a `&mut self`, ensuring nothing is using
    /// the 'frozen' contents.
    pub fn as_mut(&mut self) -> &mut IndexSet<T> {
        unsafe { &mut *self.set.get() }
    }

    // TODO add more
}

impl<T> From<IndexSet<T>> for FrozenIndexSet<T> {
    fn from(set: IndexSet<T>) -> Self {
        Self {
            set: UnsafeCell::new(set),
            in_use: Cell::new(false),
        }
    }
}

impl<T: Eq + Hash + StableDeref> Index<usize> for FrozenIndexSet<T> {
    type Output = T::Target;
    fn index(&self, idx: usize) -> &T::Target {
        assert!(!self.in_use.get());
        self.in_use.set(true);
        let ret = unsafe {
            let set = self.set.get();
            &*(*set)[idx]
        };
        self.in_use.set(false);
        ret
    }
}

impl<T: Eq + Hash> FromIterator<T> for FrozenIndexSet<T> {
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = T>,
    {
        let set: IndexSet<_> = iter.into_iter().collect();
        set.into()
    }
}

impl<T: Eq + Hash> Default for FrozenIndexSet<T> {
    fn default() -> Self {
        FrozenIndexSet::new()
    }
}
