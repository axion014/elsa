//! **This module is experimental**
//!
//! This module provides threadsafe versions of FrozenMap and FrozenVec,
//! ideal for use as a cache.
//!
//! These lock internally, however locks only last as long as the method calls
//!

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use stable_deref_trait::StableDeref;

use std::sync::RwLock;

/// Append-only threadsafe version of `std::collections::HashMap` where
/// insertion does not require mutable access
pub struct FrozenMap<K, V> {
    map: RwLock<HashMap<K, V>>,
}

impl<K: Eq + Hash, V: StableDeref> FrozenMap<K, V> {
    // these should never return &K or &V
    // these should never delete any entries

    pub fn new() -> Self {
        Self {
            map: RwLock::new(Default::default()),
        }
    }

    pub fn insert(&self, k: K, v: V) -> &V::Target {
        let mut map = self.map.write().unwrap();
        let ret = unsafe {
            let inserted = &**map.entry(k).or_insert(v);
            &*(inserted as *const _)
        };
        ret
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use elsa::sync::FrozenMap;
    ///
    /// let map = FrozenMap::new();
    /// map.insert(1, Box::new("a"));
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V::Target>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let map = self.map.read().unwrap();
        let ret = unsafe { map.get(k).map(|x| &*(&**x as *const V::Target)) };
        ret
    }

    /// Applies a function to the owner of the value corresponding to the key (if any).
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use elsa::sync::FrozenMap;
    ///
    /// let map = FrozenMap::new();
    /// map.insert(1, Box::new("a"));
    /// assert_eq!(map.map_get(&1, Clone::clone), Some(Box::new("a")));
    /// assert_eq!(map.map_get(&2, Clone::clone), None);
    /// ```
    pub fn map_get<Q: ?Sized, T, F>(&self, k: &Q, f: F) -> Option<T>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
        F: FnOnce(&V) -> T,
    {
        let map = self.map.read().unwrap();
        let ret = map.get(k).map(f);
        ret
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use elsa::sync::FrozenMap;
    ///
    /// let mut map = FrozenMap::new();
    /// map.insert(1, Box::new("a"));
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, t: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let map = self.map.read().unwrap();
        map.contains_key(t)
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // TODO add more
}

/// Append-only threadsafe version of `std::vec::Vec` where
/// insertion does not require mutable access
pub struct FrozenVec<T> {
    vec: RwLock<Vec<T>>,
}

impl<T: StableDeref> FrozenVec<T> {
    pub fn new() -> Self {
        Self {
            vec: RwLock::new(Default::default()),
        }
    }

    // these should never return &T
    // these should never delete any entries

    pub fn push(&self, val: T) {
        let mut vec = self.vec.write().unwrap();
        vec.push(val);
    }

    /// Push, immediately getting a reference to the element
    pub fn push_get(&self, val: T) -> &T::Target {
        let mut vec = self.vec.write().unwrap();
        vec.push(val);
        unsafe { &*(&**vec.get_unchecked(vec.len() - 1) as *const T::Target) }
    }

    pub fn get(&self, index: usize) -> Option<&T::Target> {
        let vec = self.vec.read().unwrap();
        unsafe { vec.get(index).map(|x| &*(&**x as *const T::Target)) }
    }

    /// Returns the number of elements in the vector, also referred to as its ‘length’.
    pub fn len(&self) -> usize {
        let vec = self.vec.read().unwrap();
        vec.len()
    }

    /// Returns true if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // TODO add more
}
