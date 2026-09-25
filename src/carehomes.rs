use std::mem;

use identity_hash::IntSet;

use crate::{
    N_CARE_LEVELS,
    agents::{
        agent_modules::tasks::{accept_task, mark_task_assigned, unassign_care_tasks_from},
        tasks::Carer,
    },
    full_model::{Model, person::Id},
    population::PopIterOrder,
    utilities::int_set_with_cap,
};

pub struct CareHomes {
    ratio: f64,
    capacity: usize,
    in_home: IntSet<Id>,
}

impl CareHomes {
    pub fn new(ratio: f64, pop_size: usize) -> Self {
        let capacity = Self::target_capacity(ratio, pop_size);
        Self {
            ratio,
            capacity,
            in_home: int_set_with_cap(capacity),
        }
    }

    fn target_capacity(ratio: f64, pop_size: usize) -> usize {
        (pop_size as f64 * ratio).ceil() as usize
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn occupied(&self) -> usize {
        self.in_home.len()
    }

    pub fn free(&self) -> usize {
        self.capacity.saturating_sub(self.occupied())
    }

    pub fn grow_for_pop_size(&mut self, pop_size: usize) {
        let new_cap = Self::target_capacity(self.ratio, pop_size);
        self.capacity = self.capacity.max(new_cap);
    }

    pub fn dead(&mut self, p_id: Id) {
        self.in_home.remove(&p_id);
    }
}

pub fn accept_new_residents(model: &mut Model, order: &PopIterOrder) {
    model.carehomes.grow_for_pop_size(model.pop.size());

    let nfree_beds = model.carehomes.free();
    let mut new = Vec::with_capacity(nfree_beds);

    let new_high_priority = model
        .pop
        .alives(order)
        .filter_map(|p| (p.care.index() == N_CARE_LEVELS - 1).then_some(p.id()))
        .take(nfree_beds);
    new.extend(new_high_priority);

    let nfree_beds = nfree_beds - new.len();
    let new_lower_priority = model
        .pop
        .alives(order)
        .filter_map(|p| (p.care.index() == N_CARE_LEVELS - 2).then_some(p.id()))
        .take(nfree_beds);
    new.extend(new_lower_priority);

    for p_id in new {
        model.carehomes.in_home.insert(p_id);
        let person = model.pop.alive_mut(p_id);
        person.task.in_care_home = true;
        assign_tasks_to_care_home(p_id, model);
    }
}

pub fn assign_tasks_to_care_home(p_id: Id, model: &mut Model) {
    unassign_care_tasks_from(p_id, model);
    let person = model.pop.alive_mut(p_id);
    let care_tasks = mem::take(&mut person.task.open_tasks);
    for t_id in care_tasks.iter() {
        mark_task_assigned(t_id, model);
        accept_task(t_id, &[], Carer::CareHome, model);
    }
}
