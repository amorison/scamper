use std::mem;

use rand::seq::IndexedRandom;

use crate::{
    ModelPars,
    agents::{
        agent_modules::kinship::siblings,
        interactions::{
            dependencies::{
                can_live_alone, set_as_guardian_dependent, set_as_independent,
                set_as_self_providing,
            },
            housing::move_to_house,
        },
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    utilities::Age,
};

pub fn process_change_1yr_deps(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive(p_id);
    if person.basic.age == Age::years(pars.work.age_independence) {
        set_as_independent(p_id, model);
    }
}

pub fn process_death_deps(p_id: Id, model: &mut Model) {
    set_as_independent(p_id, model);
    set_as_self_providing(p_id, model);

    let person = model.pop.alive_mut(p_id);
    let providees = mem::take(&mut person.dependency.providees);

    for prov_id in providees {
        let providee = model.pop.alive_mut(prov_id);
        providee.dependency.provider = None;
    }
}

fn has_valid_guardian(person: &Person, model: &Model) -> bool {
    person
        .dependency
        .guardians
        .iter()
        .any(|&id| model.pop.get(id).is_alive())
}

pub fn select_assign_guardian(person: &Person, model: &Model) -> bool {
    person.basic.alive && !can_live_alone(person) && !has_valid_guardian(person, model)
}

pub fn assign_guardian(p_id: Id, model: &mut Model, order: &PopIterOrder) -> bool {
    let mut g_id = find_family_guardian(p_id, model);
    if g_id.is_none() {
        g_id = find_other_guardian(model, order);
    }

    // Get rid of previous (possibly dead) guardians. This implies that relatives of a non-related
    // former legal guardian that are now excluded due to age won't get a chance again in the
    // future.
    let person = model.pop.alive_mut(p_id);
    person.dependency.guardians.clear();

    if let Some(guard_id) = g_id {
        adopt(guard_id, p_id, model);
        true
    } else {
        false
    }
}

fn is_potential_guardian(p_id: Id, model: &Model) -> bool {
    model
        .pop
        .get(p_id)
        .is_alive_and(|p| p.basic.age >= Age::years(18))
}

fn find_family_guardian(p_id: Id, model: &Model) -> Option<Id> {
    let person = model.pop.alive(p_id);
    let parents = person.kinship.parents();

    for &g_id in &parents {
        if is_potential_guardian(g_id, model) {
            return Some(g_id);
        }
    }

    for &g_id in &person.dependency.guardians {
        if is_potential_guardian(g_id, model) {
            return Some(g_id);
        }
    }

    // Relative of biological parents, any of those might already
    // be guardians but in that case they will be dead.
    for &parent_id in &parents {
        let parent = model.pop.get(parent_id);
        for g_id in parent.kinship().parents() {
            if is_potential_guardian(g_id, model) {
                return Some(g_id);
            }
        }
        let (full, half) = siblings(parent, model);
        for &g_id in full.union(&half) {
            if is_potential_guardian(g_id, model) {
                return Some(g_id);
            }
        }
    }

    // Possible overlap with previous, but doesn't matter.
    for &guardian_id in &person.dependency.guardians {
        let guardian = model.pop.get(guardian_id);
        for g_id in guardian.kinship().parents() {
            if is_potential_guardian(g_id, model) {
                return Some(g_id);
            }
        }
        let (full, half) = siblings(guardian, model);
        for &g_id in full.union(&half) {
            if is_potential_guardian(g_id, model) {
                return Some(g_id);
            }
        }
    }

    None
}

fn find_other_guardian(model: &mut Model, order: &PopIterOrder) -> Option<Id> {
    let candidates: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| {
            (p.is_female()
                && can_live_alone(p)
                && p.partner(model)
                    .is_some_and(|partner| p.work.is_worker() || partner.work.is_worker()))
            .then_some(p.id())
        })
        .collect();

    candidates.choose(&mut model.rng).copied()
}

fn adopt(guard_id: Id, p_id: Id, model: &mut Model) {
    let guardian = model.pop.alive(guard_id);
    let maybe_partner = guardian.kinship.partner();
    move_to_house(p_id, guardian.house, model);
    set_as_guardian_dependent(guard_id, p_id, model);
    if let Some(partner_id) = maybe_partner {
        set_as_guardian_dependent(partner_id, p_id, model);
    }
}
