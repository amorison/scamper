use std::{collections::HashSet, mem};

use crate::{
    agents::tasks::{Carer, IdTask, Task},
    full_model::{
        Model,
        person::{Id, Person},
    },
    utilities::HourInWeek,
};

// FIXME: rethink how to store assigned vs open tasks

pub struct TaskPerson {
    pub assigned_tasks: HashSet<IdTask>,
    pub open_tasks: HashSet<IdTask>,
    /// Focus per hour
    task_schedule: [[f64; 24]; 7],
    /// Tasks the agent does, sorted per day
    pub todo: [Vec<IdTask>; 7],
    pub care_task_hours: u32,
}

impl Default for TaskPerson {
    fn default() -> Self {
        Self {
            assigned_tasks: HashSet::new(),
            open_tasks: HashSet::new(),
            task_schedule: [[0.0; 24]; 7],
            todo: Default::default(),
            care_task_hours: 0,
        }
    }
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
        assert!(!self.todo[day].contains(&task_id));
        self.todo[day].push(task_id);

        if task.is_care() && self.task_schedule[day][hour] <= 0.0 {
            self.care_task_hours += 1;
        }
        self.task_schedule[day][hour] += task.focus();
        assert!(self.task_schedule[day][hour] <= 1.0);
    }

    pub fn unschedule_task(&mut self, task: &Task) {
        let (day, hour) = task.day_hour();
        let day = day.index();
        let hour = hour.index();
        let task_id = task.id();

        let idx = self.todo[day]
            .iter()
            .position(|&i| i == task_id)
            .expect("task should be in todo");
        self.todo[day].swap_remove(idx);

        self.task_schedule[day][hour] -= task.focus();

        if task.is_care() && self.task_schedule[day][hour] <= 0.0 {
            self.care_task_hours -= 1;
        }
        assert!(self.task_schedule[day][hour] >= 0.0);
    }

    pub fn how_busy_at(&self, hour: HourInWeek) -> f64 {
        let (day, hour) = hour.day_hour();
        self.task_schedule[day.index()][hour.index()]
    }

    /// Whether a task of the given focus can be accomodated by an agent's schedule without changes.
    pub fn focus_fits_in_sched(&self, hour: HourInWeek, focus: f64) -> bool {
        self.how_busy_at(hour) + focus <= 1.0
    }

    /// Whether the given task can be accomodated by an agent's schedule without changes.
    pub fn task_fits_in_sched(&self, hour: HourInWeek, task: &Task) -> bool {
        self.how_busy_at(hour) + task.focus() <= 1.0
    }
}

pub fn mark_task_assigned(task_id: IdTask, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    let owner = model.pop.alive_mut(task.owner());
    owner.task.open_tasks.remove(&task_id);
    owner.task.assigned_tasks.insert(task_id);
    task.worker = None;
}

pub fn mark_task_unassigned(task_id: IdTask, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    let owner = model.pop.alive_mut(task.owner());
    owner.task.assigned_tasks.remove(&task_id);
    owner.task.open_tasks.insert(task_id);
    task.worker = None;
}

pub fn remove_all_tasks(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    let assigned_tasks = mem::take(&mut person.task.assigned_tasks);

    // FIXME: is this also where we remove the tasks from model.tasks?

    for task_id in assigned_tasks {
        let task = model.tasks.get(&task_id).unwrap();
        let person = model.pop.alive_mut(p_id);
        person.task.unschedule_task(task);
        mark_task_unassigned(task_id, model);
    }

    let person = model.pop.alive_mut(p_id);
    person.task.assigned_tasks.clear();
}

// FIXME: this actually empties the todo list completely.
pub fn remove_all_care(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    let todo = mem::take(&mut person.task.todo);
    person.task.task_schedule = Default::default();
    person.task.care_task_hours = 0;

    for task_id in todo.into_iter().flatten() {
        mark_task_unassigned(task_id, model);
    }
}

pub fn accept_task(task_id: IdTask, tasks_to_clear: &[IdTask], carer: Carer, model: &mut Model) {
    let task = model.tasks.get_mut(&task_id).unwrap();
    task.worker = Some(carer);
    if let Carer::Person(p_id) = carer {
        let person = model.pop.alive_mut(p_id);
        person.task.schedule_task(task);

        for t_id in tasks_to_clear {
            let task = model.tasks.get_mut(t_id).unwrap();
            let person = model.pop.alive_mut(p_id);
            person.task.unschedule_task(task);
            mark_task_unassigned(*t_id, model);
        }
    }
}

pub fn find_tasks_at(person: &Person, hiw: HourInWeek, model: &Model) -> Vec<IdTask> {
    if person.task.how_busy_at(hiw) <= 0.0 {
        return Vec::new();
    }

    let (day, _) = hiw.day_hour();
    person.task.todo[day.index()]
        .iter()
        .copied()
        .filter(|t_id| model.tasks.get(t_id).unwrap().time() == hiw)
        .collect()
}
