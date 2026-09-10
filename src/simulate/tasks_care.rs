use std::mem;

use identity_hash::IntMap;
use rand::{RngExt, seq::IndexedRandom};

use crate::{
    ModelPars,
    agents::{
        agent_modules::{
            kinship::{are_siblings, siblings},
            tasks::{
                accept_task, empty_todo, find_tasks_at, mark_task_assigned, mark_task_unassigned,
                unassign_care_tasks_from,
            },
        },
        tasks::{Carer, IdTask, Task, TaskKind},
    },
    common::tasks_care::{init_care_tasks, weekly_care_supply},
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::{AliveOrDead, PopIterOrder},
    utilities::{Age, HourInWeek, int_map_with_cap},
};

pub fn process_change_1yr_task_care(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive(p_id);
    if person.basic.age == Age::years(pars.task_care.stop_baby_care_age)
        || person.basic.age == Age::years(pars.task_care.stop_child_care_age)
    {
        care_need_changed(p_id, model, pars);
    }
}

// FIXME: misnamed since now this concerns all tasks.
pub fn process_death_task_care(p_id: Id, model: &mut Model) {
    unassign_care_tasks_from(p_id, model);
    empty_todo(p_id, model);

    // this removes the tasks needed by the person.
    let person = model.pop.alive_mut(p_id);
    let open_tasks = mem::take(&mut person.task.open_tasks);
    let assigned_tasks = mem::take(&mut person.task.assigned_tasks);
    for task_id in open_tasks.into_iter().chain(assigned_tasks) {
        model.tasks.remove(&task_id);
    }

    // FIXME: older tasks that were forgotten are not removed here
}

pub fn care_need_changed(p_id: Id, model: &mut Model, pars: &ModelPars) {
    unassign_care_tasks_from(p_id, model);
    init_care_tasks(p_id, pars, model);
}

/// Try to assign cares for open care tasks.
pub fn distribute_care(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    // Tasks can be rejected, and more important tasks can override
    // already assigned tasks, so we iterate a couple of times.

    for _ in 0..pars.task_care.n_iter_care_dist {
        // collect all open tasks
        let mut asked_tasks = int_map_with_cap(model.pop.size() * 3);
        for p_id in order.ids() {
            let caree = model.pop.alive(p_id);
            if !caree.task.has_open_tasks() {
                continue;
            }
            assign_open_tasks(p_id, &mut asked_tasks, model, pars);
        }

        // let carers accept tasks
        for (carer_id, tasks) in asked_tasks {
            check_accept_tasks(carer_id, &tasks, model, pars);
        }
    }
}

fn during_school_time(task: &Task) -> bool {
    let (d, h) = task.time().day_hour();
    (0..5).contains(&d.index()) && (10..17).contains(&h.index())
}

fn is_for_school_care(task: &Task) -> bool {
    task.kind == TaskKind::ChildCare && during_school_time(task)
}

fn assign_school_care(p_id: Id, model: &mut Model) {
    let person = model.pop.alive(p_id);
    if person.basic.age < Age::years(4) || person.basic.age >= Age::years(16) {
        return;
    }
    let tasks_to_assign: Vec<_> = person
        .task
        .open_tasks
        .iter()
        .copied()
        .filter(|t_id| {
            let task = model.tasks.get(t_id).unwrap();
            is_for_school_care(task)
        })
        .collect();

    for t_id in tasks_to_assign {
        accept_task(t_id, &[], Carer::School, model);
        mark_task_assigned(t_id, model);
    }
}

/// Add tasks to carer's list of asked tasks.
fn add_asked_tasks(carer: Id, tasks: Vec<IdTask>, asked_tasks: &mut IntMap<Id, Vec<IdTask>>) {
    asked_tasks.entry(carer).or_default().extend(tasks);
}

/// Rough measure of availability, does not take into account focus.
fn available_care_time(agent: &Person, pars: &ModelPars) -> f64 {
    weekly_care_supply(agent, pars).saturating_sub(agent.task.care_task_hours) as f64
}

/// Add agent to list if minimum requirements are met.
fn check_and_add_carer(list: &mut Vec<Id>, p_id: Id, model: &Model, pars: &ModelPars) {
    // TODO: check location (max dist?)
    // FIXME: list should be a set for cheaper check?
    if !list.contains(&p_id)
        && model
            .pop
            .get(p_id)
            .is_alive_and(|agent| available_care_time(agent, pars) > 0.0)
    {
        list.push(p_id);
    }
}

/// Relatedness as a number in order: child, parent, partner, sibling, other
fn related_status(of_agent: &Person, to_agent: &Person) -> usize {
    if to_agent.kinship.children.contains(&of_agent.id()) {
        0
    } else if of_agent.kinship.children.contains(&to_agent.id()) {
        1
    } else if to_agent.kinship.partner() == Some(of_agent.id()) {
        2
    } else if are_siblings(of_agent, to_agent) {
        3
    } else {
        4
    }
}

/// Create a list of potential carers for an agent.
fn create_carer_list(agent: &Person, model: &Model, pars: &ModelPars) -> Vec<Id> {
    let mut potential_carers = Vec::new();

    for &id_occ in agent.house(model).basic.occupants() {
        check_and_add_carer(&mut potential_carers, id_occ, model, pars);
    }

    for parent_id in agent.kinship.parents() {
        check_and_add_carer(&mut potential_carers, parent_id, model, pars);
    }

    for &child_id in &agent.kinship.children {
        check_and_add_carer(&mut potential_carers, child_id, model, pars);
    }

    for sibling_id in siblings(AliveOrDead::Alive(agent), model) {
        check_and_add_carer(&mut potential_carers, sibling_id, model, pars);
    }

    potential_carers
}

fn care_weight_distance(carer: &Person, caree: &Person, model: &Model, pars: &ModelPars) -> f64 {
    if carer.house == caree.house {
        pars.task_care.care_weight_distance[0]
    } else if carer.house(model).basic.town() == caree.house(model).basic.town() {
        pars.task_care.care_weight_distance[1]
    } else {
        pars.task_care.care_weight_distance[2]
    }
}

/// Preference for a given potential carer dependent on task type.
fn task_ask_weight(
    potential_carer: &Person,
    caree: &Person,
    task_kind: TaskKind,
    model: &Model,
    pars: &ModelPars,
) -> f64 {
    let mut weight = care_weight_distance(potential_carer, caree, model, pars);

    let rel = related_status(potential_carer, caree);
    weight *= pars.task_care.care_weight_related[rel][task_kind.weight_index()];

    weight
}

/// Return all open tasks of the given kind at a randomly selected day.
fn get_chunk_of_open_tasks(p_id: Id, task_kind: TaskKind, model: &mut Model) -> Vec<IdTask> {
    let agent = model.pop.alive_mut(p_id);

    // Can be simplified with `collect_into` once it is stabilised.
    let mut tasks_of_kind = Vec::with_capacity(agent.task.open_tasks.len());
    let tasks_iter = agent.task.open_tasks.iter().filter_map(|t_id| {
        let task = model.tasks.get(t_id).unwrap();
        (task.kind == task_kind).then_some((task.id(), task.time()))
    });
    tasks_of_kind.extend(tasks_iter);

    let tasks = if let Some(&(_, time)) = tasks_of_kind.choose(&mut model.rng) {
        let mut tasks_at_time = Vec::with_capacity(tasks_of_kind.len());
        let tasks_at_time_iter = tasks_of_kind
            .into_iter()
            .filter_map(|(id, t)| (time == t).then_some(id));
        tasks_at_time.extend(tasks_at_time_iter);
        tasks_at_time
    } else {
        Vec::new()
    };

    for &t_id in &tasks {
        assert!(agent.task.assigned_tasks.insert(t_id));
        assert!(agent.task.open_tasks.remove(&t_id));
    }

    tasks
}

// FIXME: this should be different for formal care
fn availability_weight(carer: &Person, tasks: &[IdTask], pars: &ModelPars) -> f64 {
    let w = available_care_time(carer, pars) / tasks.len() as f64;
    debug_assert!(w >= 0.0);
    w + 1.0
}

/// Assign all open tasks of an agent to a potential carer.
fn assign_open_tasks(
    p_id: Id,
    asked_tasks: &mut IntMap<Id, Vec<IdTask>>,
    model: &mut Model,
    pars: &ModelPars,
) {
    let agent = model.pop.alive(p_id);
    if !agent.task.has_open_tasks() {
        return;
    }

    let potential_carers = create_carer_list(agent, model, pars);

    assign_school_care(p_id, model);

    for tt in [TaskKind::ChildCare, TaskKind::SocialCare] {
        let agent = model.pop.alive(p_id);
        let tt_weights: Vec<_> = potential_carers
            .iter()
            .map(|&carer_id| {
                let potential_carer = model.pop.alive(carer_id);
                task_ask_weight(potential_carer, agent, tt, model, pars)
            })
            .collect();

        if tt_weights.iter().sum::<f64>() <= 0.0 {
            // nobody available for this task type
            continue;
        }

        // get a chunk of tasks of the same type (specifically with the same
        // weight calculation)
        // We stop (and continue with next tt) when there are no more tasks of this type.
        loop {
            let tasks = get_chunk_of_open_tasks(p_id, tt, model);
            if tasks.is_empty() {
                break;
            }

            // preferentially ask those that are available most of the time
            let weights: Vec<_> = potential_carers
                .iter()
                .enumerate()
                .map(|(i, &p_carer)| {
                    let carer = model.pop.alive(p_carer);
                    tt_weights[i] * availability_weight(carer, &tasks, pars)
                })
                .collect();

            // draw agent to ask
            let indices: Vec<_> = (0..potential_carers.len()).collect();
            let &i_carer = indices
                .choose_weighted(&mut model.rng, |&i| weights[i])
                .unwrap();
            let p_carer = potential_carers[i_carer];

            add_asked_tasks(p_carer, tasks, asked_tasks);
        }
    }
}

/// Importance of the task to the carer.
fn task_importance(task: &Task, agent: &Person, model: &Model, pars: &ModelPars) -> f64 {
    let owner = model.pop.alive(task.owner);
    let rel = related_status(agent, owner);
    pars.task_care.care_weight_related[rel][task.kind.weight_index()] * task.urgency
}

// FIXME: stale comment: work is tasks now
/// Get importance of current tasks at a given hour. Returns list of priorities and list of tasks,
/// an empty array if there are none, or nothing if agent has to work.
fn get_importance_at(
    agent: &Person,
    hiw: HourInWeek,
    model: &Model,
    pars: &ModelPars,
) -> Vec<(IdTask, f64)> {
    find_tasks_at(agent, hiw, model)
        .into_iter()
        .map(|t_id| {
            let task = model.tasks.get(&t_id).unwrap();
            (t_id, task_importance(task, agent, model, pars))
        })
        .collect()
}

/// Get list of tasks that would have to be given up in order to accomodate new task.
fn task_accept_plan(
    agent: &Person,
    task: &Task,
    model: &Model,
    pars: &ModelPars,
) -> Vec<(IdTask, f64)> {
    let mut tasks = get_importance_at(agent, task.time(), model, pars);

    if tasks.is_empty() {
        return tasks;
    }

    // FIXME: Julia has `tasks.is_empty() &&` here. This should probably be `!is_empty`, which is
    // always true at this point in the Rust code anyway.
    // tapped out, can't give up free hour
    if available_care_time(agent, pars) <= 0.0 {
        return tasks;
    }

    // If new task is at a different location we'd have to give up all previous
    // tasks. Currently assumes caree determines location.
    let task0 = model.tasks.get(&tasks[0].0).unwrap();
    if !task.is_colocated(task0) {
        return tasks;
    }

    // Sort by importance
    tasks.sort_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap());

    let mut f = task.focus();
    // Keep as many important tasks as possible.
    while let Some(&(t_id, _)) = tasks.last() {
        let most_imp_task = model.tasks.get(&t_id).unwrap();
        f += most_imp_task.focus();
        if f > 1.0 {
            break;
        }
        tasks.pop();
    }

    tasks
}

/// Sigmoid such that f(0) = 0, f(1) = 1, f(1/2) = 1/2.
/// Linear for shape = 1.0, higher values increase slope at 0.5.
fn sigmoid(x: f64, shape: f64) -> f64 {
    let xs = x.powf(shape);
    xs / (xs + (1.0 - x).powf(shape))
}

/// Probability that a task gets accepted.
fn task_accept_prob(
    task: &Task,
    give_up: &[(IdTask, f64)],
    carer: &Person,
    model: &Model,
    pars: &ModelPars,
) -> f64 {
    let importance = task_importance(task, carer, model, pars);
    let cur_importance = 1.0 - give_up.iter().map(|gu| 1.0 - gu.1).product::<f64>();

    let ratio = importance / (importance + cur_importance);

    sigmoid(ratio, pars.task_care.accept_prob_polarity)
    // Comment in Julia:
    // TODO: rethink, this doesn't make sense
    // should modify the shape of sigmoid instead
    // maybe also introduce reluctance to switch task
    // diligence(carer, sigmoid * importance)
    // With diligence defined as
    // importance.powf(agent.task.diligence)
}

/// Check for all tasks whether to accept.
fn check_accept_tasks(p_id: Id, tasks: &[IdTask], model: &mut Model, pars: &ModelPars) {
    for &t_id in tasks {
        let agent = model.pop.alive(p_id);
        let task = model.tasks.get(&t_id).unwrap();
        let tasks_to_give_up = task_accept_plan(agent, task, model, pars);
        let prob = task_accept_prob(task, &tasks_to_give_up, agent, model, pars);

        if model.rng.random_bool(prob) {
            let tasks_to_clear: Vec<_> = tasks_to_give_up.into_iter().map(|gu| gu.0).collect();
            accept_task(t_id, &tasks_to_clear, Carer::Person(p_id), model);
        } else {
            mark_task_unassigned(t_id, model);
        }
    }
}
