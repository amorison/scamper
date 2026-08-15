use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    full_model::person::Id,
    utilities::{DayInWeek, HourInDay, HourInWeek},
};

static ID_TASK: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdTask(u64);

impl IdTask {
    fn new() -> IdTask {
        IdTask(ID_TASK.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    ChildCare,
    SocialCare,
    Work,
}

impl TaskKind {
    pub fn is_care(self) -> bool {
        match self {
            TaskKind::ChildCare => true,
            TaskKind::SocialCare => true,
            TaskKind::Work => false,
        }
    }

    pub fn weight_index(self) -> usize {
        match self {
            TaskKind::ChildCare => 0,
            TaskKind::SocialCare => 1,
            TaskKind::Work => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Carer {
    School,
    Person(Id),
}

pub struct Task {
    id: IdTask,
    pub kind: TaskKind,
    pub owner: Id,
    pub worker: Option<Carer>,
    pub time: HourInWeek,
    pub urgency: f64,
    pub focus: f64,
}

impl Task {
    pub fn child_care(owner: Id, time: HourInWeek) -> Self {
        Self {
            id: IdTask::new(),
            kind: TaskKind::ChildCare,
            owner,
            worker: None,
            time,
            urgency: 0.5,
            focus: 0.5,
        }
    }

    pub fn social_care(owner: Id, time: HourInWeek) -> Self {
        Self {
            id: IdTask::new(),
            kind: TaskKind::SocialCare,
            owner,
            worker: None,
            time,
            urgency: 0.5,
            focus: 0.5,
        }
    }

    pub fn work(owner: Id, time: HourInWeek) -> Self {
        Self {
            id: IdTask::new(),
            kind: TaskKind::Work,
            owner,
            worker: Some(Carer::Person(owner)),
            time,
            urgency: 1.0,
            focus: 1.0,
        }
    }

    pub fn id(&self) -> IdTask {
        self.id
    }

    pub fn owner(&self) -> Id {
        self.owner
    }

    pub fn is_care(&self) -> bool {
        self.kind.is_care()
    }

    pub fn time(&self) -> HourInWeek {
        self.time
    }

    pub fn day_hour(&self) -> (DayInWeek, HourInDay) {
        self.time.day_hour()
    }

    pub fn focus(&self) -> f64 {
        self.focus
    }

    pub fn is_colocated(&self, other: &Task) -> bool {
        self.owner == other.owner
    }
}
