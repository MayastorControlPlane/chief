use common_lib::types::v0::store::UuidString;
use parking_lot::Mutex;
use std::{collections::HashMap, hash::Hash, sync::Arc};

#[derive(Default, Debug)]
pub struct ResourceMap<I, S> {
    map: HashMap<I, Arc<Mutex<S>>>,
}

impl<I, S> ResourceMap<I, S>
where
    I: Eq + Hash + From<String>,
    S: Clone + UuidString,
{
    /// Get the resource with the given key.
    pub fn get(&self, key: &I) -> Option<&Arc<Mutex<S>>> {
        self.map.get(key)
    }

    /// Clear the contents of the map.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Insert an element or update an existing entry in the map.
    pub fn insert(&mut self, key: I, value: Arc<Mutex<S>>) {
        self.map.insert(key, value);
    }

    /// Remove an element from the map.
    pub fn remove(&mut self, key: &I) {
        self.map.remove(key);
    }

    /// Update the resource map.
    pub fn update(&mut self, values: Vec<S>) {
        // First clear the map. This ensures any stale entries (not in the vector of values) will be
        // removed.
        self.clear();
        for value in values {
            self.map
                .insert(value.uuid_as_string().into(), Arc::new(Mutex::new(value)));
        }
    }

    /// Get all the resources as a vector.
    pub fn to_vec(&self) -> Vec<Arc<Mutex<S>>> {
        self.map.values().cloned().collect()
    }
}
