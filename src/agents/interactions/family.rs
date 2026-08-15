use std::debug_assert_matches;

use crate::{
    agents::agent_modules::{
        basic_info::Gender,
        kinship::{Partnership, are_parent_child, are_siblings},
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    utilities::Age,
};

pub fn related_1st_degree(p1: &Person, p2: &Person) -> bool {
    are_parent_child(p1, p2) || are_siblings(p1, p2)
}

pub fn set_as_parent_child(child_id: Id, parent_id: Id, model: &mut Model) {
    let parent = model.population.get_mut(&parent_id).unwrap();
    let age_parent = parent.basic.age;
    let gender_parent = parent.basic.gender;
    parent.kinship.add_child(child_id);

    let child = model.population.get_mut(&child_id).unwrap();
    debug_assert!(child.basic.age < age_parent);
    match gender_parent {
        Gender::Female => {
            assert!(child.kinship.mother.is_none());
            child.kinship.mother = Some(parent_id);
        }
        Gender::Male => {
            assert!(child.kinship.father.is_none());
            child.kinship.father = Some(parent_id);
        }
    }
}

pub fn reset_partner(p_id: Id, model: &mut Model) {
    let person = model.population.get_mut(&p_id).unwrap();
    if let Some(partnership) = person.kinship.partnership.take() {
        let partner_id = partnership.with();
        let partner = model.population.get_mut(&partner_id).unwrap();
        partner.kinship.partnership = None;
    }
}

pub fn resolve_partnership(p1_id: Id, p2_id: Id, model: &mut Model) {
    let p1 = model.population.get(&p1_id).unwrap();
    let p2 = model.population.get(&p2_id).unwrap();
    assert!(p1.kinship.partner() == Some(p2_id) && p2.kinship.partner() == Some(p1_id));
    reset_partner(p1_id, model);
}

pub fn set_as_partners(p1_id: Id, p2_id: Id, model: &mut Model) {
    reset_partner(p1_id, model);
    reset_partner(p2_id, model);

    let p1 = model.population.get_mut(&p1_id).unwrap();
    debug_assert_matches!(p1.basic.gender, Gender::Male);
    p1.kinship.partnership = Some(Partnership::new(p2_id));

    let p2 = model.population.get_mut(&p2_id).unwrap();
    debug_assert_matches!(p2.basic.gender, Gender::Female);
    p2.kinship.partnership = Some(Partnership::new(p1_id));
}

pub fn has_alive_child(parent: &Person, model: &Model) -> bool {
    parent
        .kinship
        .children
        .iter()
        .any(|c_id| model.population.get(c_id).unwrap().basic.alive)
}

pub fn has_young_infant(parent: &Person, model: &Model) -> bool {
    parent.kinship.children.iter().any(|c_id| {
        let child = model.population.get(c_id).unwrap();
        child.basic.alive && child.basic.age <= Age::years(1)
    })
}

// FIXME: slightly misnamed, this checks if at least one child is home.
pub fn has_own_children_at_home(parent: &Person, model: &Model) -> bool {
    parent.kinship.children.iter().any(|c_id| {
        let child = model.population.get(c_id).unwrap();
        child.basic.alive && child.house == parent.house
    })
}
