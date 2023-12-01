//! support for snapshotting different states

use alloy_primitives::U256;
use std::{collections::HashMap, ops::Add};

/// Represents all snapshots
#[derive(Debug, Clone)]
pub struct Snapshots<T> {
    id: U256,
    snapshots: HashMap<U256, T>,
}

impl<T: Clone> Snapshots<T> {
    fn next_id(&mut self) -> U256 {
        let id = self.id;
        self.id = id.saturating_add(U256::from(1));
        id
    }

    /// Returns the snapshot with the given id `id`
    pub fn get(&self, id: U256) -> Option<&T> {
        self.snapshots.get(&id)
    }

    /// Removes the snapshot with the given `id`.
    ///
    /// This will also remove any snapshots taken after the snapshot with the `id`. e.g.: reverting
    /// to id 1 will delete snapshots with ids 1, 2, 3, etc.)
    pub fn remove(&mut self, id: U256) -> Option<T> {
        let snapshot = self.snapshots.remove(&id);

        // revert all snapshots taken after the snapshot
        let mut to_revert = id.add(U256::from(1));
        while to_revert < self.id {
            self.snapshots.remove(&to_revert);
            to_revert += U256::from(1);
        }

        snapshot
    }

    /// Removes all snapshots after `id`.
    ///
    /// This will not remove the `id` itself, only clone its data, since the snapshot will still
    /// point to a valid state. e.g.: reverting
    /// to id 1 will delete snapshots with ids 2, 3, etc.)
    pub fn revert_to(&mut self, id: U256) -> Option<T> {
        let snapshot = self.snapshots.get(&id).cloned();

        // revert all snapshots taken after the snapshot
        self.snapshots.retain(|k, _| *k <= id);

        snapshot
    }

    /// Removes the snapshot with the given `id`.
    ///
    /// Does not remove snapshots after it.
    pub fn remove_at(&mut self, id: U256) -> Option<T> {
        self.snapshots.remove(&id)
    }

    /// Inserts the new snapshot and returns the id
    pub fn insert(&mut self, snapshot: T) -> U256 {
        let id = self.next_id();
        self.snapshots.insert(id, snapshot);
        id
    }

    /// Inserts the new snapshot at the given `id`.
    ///
    ///  Does not auto-increment the next `id`.
    pub fn insert_at(&mut self, snapshot: T, id: U256) -> U256 {
        self.snapshots.insert(id, snapshot);
        id
    }
}

impl<T> Default for Snapshots<T> {
    fn default() -> Self {
        Self { id: U256::ZERO, snapshots: HashMap::new() }
    }
}
