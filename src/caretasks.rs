use std::mem;

use crate::{
    agents::tasks::{IdTask, Task, TaskKind},
    utilities::DayInWeek,
};

struct TasksPerDay([Vec<IdTask>; 7]);

impl TasksPerDay {
    const DFLT_CAP: usize = 16;

    fn is_empty(&self) -> bool {
        self.0.iter().all(|v| v.is_empty())
    }

    fn insert_id(&mut self, t_id: IdTask, iday: usize) {
        let slot = &mut self.0[iday];
        if !slot.contains(&t_id) {
            slot.push(t_id);
        }
    }

    fn insert_ids(&mut self, t_ids: &[IdTask], iday: usize) {
        t_ids
            .iter()
            .copied()
            .for_each(|t_id| self.insert_id(t_id, iday));
    }

    fn insert(&mut self, task: &Task) {
        self.insert_id(task.id(), task.time().day_idx());
    }

    fn remove(&mut self, task: &Task) {
        let iday = task.time().day_idx();
        let t_id = task.id();
        let slot = &mut self.0[iday];
        if let Some(i) = slot.iter().position(|&t| t_id == t) {
            slot.swap_remove(i);
        }
    }

    fn take_at_day(&mut self, day: DayInWeek) -> Vec<IdTask> {
        let mut new = Vec::with_capacity(Self::DFLT_CAP);
        mem::swap(&mut new, &mut self.0[day.index()]);
        new
    }

    fn iter(&self) -> impl Iterator<Item = IdTask> {
        self.0.iter().flatten().copied()
    }

    fn drain(&mut self) -> impl Iterator<Item = IdTask> {
        self.0.iter_mut().flat_map(|v| v.drain(..))
    }
}

impl Default for TasksPerDay {
    fn default() -> Self {
        let inner = std::array::from_fn(|_| Vec::with_capacity(Self::DFLT_CAP));
        TasksPerDay(inner)
    }
}

/// Collection of care tasks.
#[derive(Default)]
pub struct CareTasks {
    child: TasksPerDay,
    health: TasksPerDay,
}

impl CareTasks {
    pub fn is_empty(&self) -> bool {
        self.child.is_empty() && self.health.is_empty()
    }

    pub fn ntasks_on(&self, task_kind: TaskKind, day: DayInWeek) -> usize {
        match task_kind {
            TaskKind::ChildCare => self.child.0[day.index()].len(),
            TaskKind::SocialCare => self.health.0[day.index()].len(),
            TaskKind::Work => panic!("only care tasks are handled"),
        }
    }

    pub fn transfer_to_and_copy(
        &mut self,
        task_kind: TaskKind,
        day: DayInWeek,
        other: &mut CareTasks,
    ) -> Vec<IdTask> {
        let tasks = match task_kind {
            TaskKind::ChildCare => self.child.take_at_day(day),
            TaskKind::SocialCare => self.health.take_at_day(day),
            TaskKind::Work => panic!("only care tasks are handled"),
        };

        let iday = day.index();
        match task_kind {
            TaskKind::ChildCare => other.child.insert_ids(&tasks, iday),
            TaskKind::SocialCare => other.health.insert_ids(&tasks, iday),
            TaskKind::Work => panic!("only care tasks are handled"),
        }

        tasks
    }

    pub fn remove(&mut self, task: &Task) {
        match task.kind {
            TaskKind::ChildCare => self.child.remove(task),
            TaskKind::SocialCare => self.health.remove(task),
            TaskKind::Work => panic!("only care tasks are handled"),
        }
    }

    pub fn insert(&mut self, task: &Task) {
        match task.kind {
            TaskKind::ChildCare => self.child.insert(task),
            TaskKind::SocialCare => self.health.insert(task),
            TaskKind::Work => panic!("only care tasks are handled"),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = IdTask> {
        self.child.iter().chain(self.health.iter())
    }

    pub fn drain(&mut self) -> impl Iterator<Item = IdTask> {
        self.child.drain().chain(self.health.drain())
    }
}
