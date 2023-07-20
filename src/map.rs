use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use dashmap::mapref::one::RefMut;
use dashmap::DashMap;

/// A type that can be used as a key in a [`Map`].
pub trait Key: Clone + Debug + Eq + Hash {}

impl<T: Clone + Debug + Eq + Hash> Key for T {}

/// A new-type over [`Arc<DashMap<K, V>>`] that abstracts items manipulation.
///
/// This type provides a method to get a "slot" ([`MapSlot<K, V>`]), which is
/// associated with an item in the map, and will remove it from the map when
/// dropped.
#[derive(Debug, Clone)]
pub struct Map<K: Key, V>(Arc<DashMap<K, V>>);

/// A slot that is associated with an item in a [`Map`]. When dropped, the
/// item associated with this slot is removed from the map.
///
/// A reference to the inner map pointer of [`Map`] is maintained to allow
/// moving the slot around more freely.
#[derive(Debug)]
pub struct MapSlot<K: Key, V> {
    map: Arc<DashMap<K, V>>,
    key: K,
}

impl<K: Key, V> Map<K, V> {
    /// Creates an new instance of [`Map`].
    #[inline]
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Gets a **mutable** reference to the item identified with `key`.
    ///
    /// # Parameters
    /// * `key` - The key identifying the item to get.
    ///
    /// # Returns
    /// * [`Some(RefMut<'_, K, V>)`] - A mutable reference to the item if found.
    /// * [`None`] - If no item is found.
    #[inline]
    pub fn get_mut(&self, key: &K) -> Option<RefMut<'_, K, V>> {
        self.0.get_mut(key)
    }

    /// Inserts a new item with `key` and `value` into the map if no existing
    /// item is found with the same `key`; otherwise, replaces the existing
    /// value with `value`.
    ///
    /// # Parameters
    /// * `key` - The key identifying the item to insert or replace.
    /// * `value` - The value to insert or replace.
    ///
    /// # Returns
    /// * [`MapSlot<K, V>`] - An object that removes the inserted or replaced
    ///  item from the map when dropped.
    #[inline]
    pub fn get_or_insert<F>(&self, key: K, with: F) -> RefMut<'_, K, V>
    where
        F: FnOnce(MapSlot<K, V>) -> V,
    {
        self.0
            .entry(key.clone())
            .or_insert_with(|| with(MapSlot::new(self.0.clone(), key)))
    }

    /// Removes the item identified with `key` from the map.
    ///
    /// # Parameters
    /// * `key` - The key identifying the item to remove.
    #[inline]
    pub fn remove(&self, key: &K) -> Option<(K, V)> {
        self.0.remove(key)
    }

    /// Clears the map, removing all items.
    #[inline]
    pub fn clear(&self) {
        self.0.clear();
    }

    /// Gets the number of items in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets whether the map is empty or not.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Key,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A handle that is associated with an item in a [`Map`]. When dropped, the
/// item associated with this handle is removed from the map.
impl<K: Key, V> MapSlot<K, V> {
    /// Creates a new instance of [`MapSlot`].
    #[inline]
    fn new(map: Arc<DashMap<K, V>>, key: K) -> Self {
        Self { map, key }
    }

    /// Gets a reference to the item associated with this slot.
    #[inline]
    pub const fn key(&self) -> &K {
        &self.key
    }

    /// Gets whether the item associated with this slot is still in the map or
    /// not.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.map.contains_key(&self.key)
    }
}

impl<K: Key, V> Drop for MapSlot<K, V> {
    #[inline]
    fn drop(&mut self) {
        self.map.remove(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn map_test() {
        let map = Map::<u32, String>::new();

        assert!(map.is_empty());

        let key: u32 = Faker.fake();
        let value: String = Faker.fake();

        let mut slot = None;
        let entry = map.get_or_insert(key, |s| {
            assert_eq!(s.key(), &key);

            slot = Some(s);
            value.clone()
        });

        assert_eq!(entry.key(), &key);
        assert_eq!(entry.value(), &value);

        drop(entry);

        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(slot.is_some());

        map.get_or_insert(key, |_| panic!("this should never happen!"));
        slot = None;

        assert!(matches!(map.get_mut(&key), None));
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.get_or_insert(key, |s| {
            assert_eq!(s.key(), &key);

            slot = Some(s);
            value.clone()
        });

        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);

        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        drop(slot);

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }
}
