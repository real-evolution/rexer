use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use dashmap::mapref::entry::Entry;
use dashmap::mapref::one::{Ref, RefMut};
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

    /// Gets a reference to the item identified with `key`.
    ///
    /// # Parameters
    /// * `key` - The key identifying the item to get.
    ///
    /// # Returns
    /// * [`Some(Ref<'_, K, V>)`] - A reference to the item if found.
    /// * [`None`] - If no item is found.
    #[inline]
    pub fn get(&self, key: &K) -> Option<Ref<'_, K, V>> {
        self.0.get(key)
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
    pub fn insert_or_replace(&self, key: K, value: V) -> MapSlot<K, V> {
        match self.0.entry(key.clone()) {
            | Entry::Occupied(o) => {
                o.replace_entry(value);
                MapSlot::new(self.0.clone(), key)
            }
            | Entry::Vacant(v) => {
                v.insert(value);
                MapSlot::new(self.0.clone(), key)
            }
        }
    }

    /// Clears the map, removing all items.
    #[inline]
    pub fn clear(&self) {
        self.0.clear();
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
