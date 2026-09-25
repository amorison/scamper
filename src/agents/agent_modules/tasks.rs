use std::mem;

use crate::{
    agents::tasks::{Carer, IdTask, Task, TaskKind},
    caretasks::CareTasks,
    full_model::{
        Model,
        person::{Id, Person},
    },
    utilities::HourInWeek,
};

#[derive(Default)]
pub struct TaskTally {
    pub child_care: u32,
    pub social_care: u32,
    pub work: u32,
}

#[derive(Default)]
pub struct TaskPerson {
    pub assigned_tasks: CareTasks,
    pub open_tasks: CareTasks,
    /// Focus per hour
    task_schedule: [[f64; 24]; 7],
    /// Tasks the agent does, sorted per day
    pub todo: [Vec<(IdTask, HourInWeek)>; 7],
    pub todo_tally: TaskTally,
    pub care_task_hours: u32,
    pub in_care_home: bool,
}

impl TaskPerson {
    pub fn has_open_tasks(&self) -> bool {
        !self.open_tasks.is_empty()
    }

    pub fn schedule_task(&mut self, task: &Task) {
        assert!(task.focus() > 0.0);
        let (day, hour) = task.day_hour();
        let day = day.index();
        let hour = hour.index();
        let task_id = task.id();
        assert!(!self.todo[day].iter().any(|&(t_id, _)| t_id == task_id));
        self.todo[day].push((task_id, task.time()));

        match task.kind {
            TaskKind::ChildCare => self.todo_tally.child_care += 1,
            TaskKind::SocialCare => self.todo_tally.social_care += 1,
            TaskKind::Work => self.todo_tally.work += 1,
        }

        if task.is_care() && self.task_schedule[day][hour] <= 0.0 {
            self.care_task_hours += 1;
        }
        self.task_schedule[day][hour] += task.focus();
        assert!(
            self.task_schedule[day][hour] <= 1.0,
            "{} == {:.} > 1.0",
            stringify!(self.task_schedule[day][hour]),
            self.task_schedule[day][hour]
        );
    }

    pub fn unschedule_task(&mut self, task: &Task) {
        self.unschedule_task_if_present(task)
            .expect("task should be in todo");
    }

    pub fn unschedule_task_if_present(&mut self, task: &Task) -> Option<()> {
        let (day, hour) = task.day_hour();
        let day = day.index();
        let hour = hour.index();
        let task_id = task.id();

        let idx = self.todo[day].iter().position(|&(i, _)| i == task_id)?;
        self.todo[day].swap_remove(idx);

        self.task_schedule[day][hour] -= task.focus();

        match task.kind {
            TaskKind::ChildCare => self.todo_tally.child_care -= 1,
            TaskKind::SocialCare => self.todo_tally.social_care -= 1,
            TaskKind::Work => self.todo_tally.work -= 1,
        }

        if task.is_care() && self.task_schedule[day][hour] <= 0.0 {
            self.care_task_hours -= 1;
        }
        assert!(self.task_schedule[day][hour] >= 0.0);

        Some(())
    }

    pub fn how_busy_at(&self, hour: HourInWeek) -> f64 {
        let (day, hour) = hour.day_hour();
        self.task_schedule[day.index()][hour.index()]
    }
}

pub fn mark_task_assigned(task_id: IdTask, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    let owner = model.pop.alive_mut(task.owner());
    owner.task.open_tasks.remove(task);
    owner.task.assigned_tasks.insert(task);
    task.worker = None;
}

pub fn mark_task_unassigned(task_id: IdTask, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    if task.is_care() {
        // FIXME: make it impossible to have work tasks going
        // into the assigned_tasks/open_tasks logic.
        let owner = model.pop.alive_mut(task.owner());
        owner.task.assigned_tasks.remove(task);
        owner.task.open_tasks.insert(task);
    }
    task.worker = None;
}

/// Unassign all care tasks currently assigned from this agent.
pub fn unassign_care_tasks_from(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    let assigned_tasks = mem::take(&mut person.task.assigned_tasks);

    // FIXME: is this also where we remove the tasks from model.tasks?

    for task_id in assigned_tasks.iter() {
        let task = model.tasks.get(&task_id).unwrap();
        // FIXME: clarify whether this check should hold
        // let Some(carer) = task.worker else {
        //     panic!("assigned_tasks should have a worker")
        // };
        if let Some(Carer::Person(w_id)) = task.worker {
            let worker = model.pop.alive_mut(w_id);
            worker.task.unschedule_task(task);
        }
        mark_task_unassigned(task_id, model);
    }
}

/// Empty to-do list and mark tasks as unassigned.
pub fn empty_todo(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    let todo = mem::take(&mut person.task.todo);
    person.task.task_schedule = Default::default();
    person.task.care_task_hours = 0;

    for (task_id, _) in todo.into_iter().flatten() {
        mark_task_unassigned(task_id, model);
    }
}

pub fn accept_task(task_id: IdTask, tasks_to_clear: &[IdTask], carer: Carer, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    task.worker = Some(carer);
    if let Carer::Person(p_id) = carer {
        for t_id in tasks_to_clear {
            let task = model.tasks.get_mut(t_id).unwrap();
            let person = model.pop.alive_mut(p_id);
            person.task.unschedule_task(task);
            mark_task_unassigned(*t_id, model);
        }

        let task = model.tasks.get(&task_id).unwrap();
        let person = model.pop.alive_mut(p_id);
        person.task.schedule_task(task);
    }
}

pub fn find_tasks_at(person: &Person, hiw: HourInWeek) -> Vec<IdTask> {
    if person.how_busy_at(hiw) <= 0.0 {
        return Vec::new();
    }

    person.task.todo[hiw.day_idx()]
        .iter()
        .copied()
        .filter_map(|(t_id, time)| (time == hiw).then_some(t_id))
        .collect()
}
