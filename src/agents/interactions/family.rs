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
    let parent = model.pop.alive_mut(parent_id);
    let age_parent = parent.basic.age;
    let gender_parent = parent.basic.gender;
    parent.kinship.add_child(child_id);

    let child = model.pop.alive_mut(child_id);
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
    let person = model.pop.alive_mut(p_id);
    if let Some(partnership) = person.kinship.partnership.take() {
        let partner_id = partnership.with();
        let partner = model.pop.alive_mut(partner_id);
        partner.kinship.partnership = None;
    }
}

pub fn resolve_partnership(p1_id: Id, p2_id: Id, model: &mut Model) {
    let p1 = model.pop.alive(p1_id);
    let p2 = model.pop.alive(p2_id);
    assert!(p1.kinship.partner() == Some(p2_id) && p2.kinship.partner() == Some(p1_id));
    reset_partner(p1_id, model);
}

pub fn set_as_partners(p1_id: Id, p2_id: Id, model: &mut Model) {
    reset_partner(p1_id, model);
    reset_partner(p2_id, model);

    let p1 = model.pop.alive_mut(p1_id);
    debug_assert!(p1.is_male());
    p1.kinship.partnership = Some(Partnership::new(p2_id));

    let p2 = model.pop.alive_mut(p2_id);
    debug_assert!(p2.is_female());
    p2.kinship.partnership = Some(Partnership::new(p1_id));
}

pub fn has_young_infant(parent: &Person, model: &Model) -> bool {
    parent.kinship.children.iter().any(|&c_id| {
        model
            .pop
            .get(c_id)
            .is_alive_and(|c| c.basic.age <= Age::years(1))
    })
}

// FIXME: slightly misnamed, this checks if at least one child is home.
pub fn has_own_children_at_home(parent: &Person, model: &Model) -> bool {
    parent.kinship.children.iter().any(|&c_id| {
        model
            .pop
            .get(c_id)
            .is_alive_and(|c| c.house == parent.house)
    })
}
