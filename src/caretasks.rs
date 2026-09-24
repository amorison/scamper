use std::collections::hash_set;

use identity_hash::IntSet;

use crate::{agents::tasks::IdTask, utilities::int_set_with_cap};

/// Collection of care tasks.
#[derive(Default)]
pub struct CareTasks(IntSet<IdTask>);

impl CareTasks {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(int_set_with_cap(capacity))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn remove(&mut self, t_id: IdTask) -> bool {
        self.0.remove(&t_id)
    }

    pub fn insert(&mut self, t_id: IdTask) -> bool {
        self.0.insert(t_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &IdTask> {
        self.0.iter()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = IdTask> {
        self.0.drain()
    }
}

impl IntoIterator for CareTasks {
    type Item = IdTask;
    type IntoIter = hash_set::IntoIter<IdTask>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
