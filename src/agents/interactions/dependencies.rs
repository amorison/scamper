use std::{cmp, mem, ops::Not};

use crate::{
    full_model::{
        Model,
        person::{Id, Person},
    },
    utilities::Age,
};

/// Whether the person shares their house with a non-dependent, non-guardian. Note that this
/// includes spouses and spouses' children.
pub fn lives_in_shared_house(p_id: Id, model: &Model) -> bool {
    let p = model.population.get(&p_id).unwrap();
    model
        .houses
        .get(&p.house)
        .unwrap()
        .basic
        .occupants()
        .iter()
        .all(|id| {
            p_id == *id
                || p.dependency.guardians.contains(id)
                || p.dependency.dependents.contains(id)
        })
        .not()
}

pub fn can_live_alone(p: &Person) -> bool {
    p.basic.age >= Age::years(18)
}

pub fn is_orphan(p: &Person) -> bool {
    !can_live_alone(p) && !p.dependency.is_dependent()
}

pub fn set_as_guardian_dependent(guardian: Id, dependent: Id, model: &mut Model) {
    let g = model.population.get_mut(&guardian).unwrap();
    g.dependency.dependents.push(dependent);
    let g_class_rank = g.class.rank;

    let d = model.population.get_mut(&dependent).unwrap();
    d.dependency.guardians.push(guardian);
    d.class.parent_rank = cmp::max(d.class.parent_rank, g_class_rank);
}

pub fn resolve_dependency(guardian: Id, dependent: Id, model: &mut Model) {
    let g = model.population.get_mut(&guardian).unwrap();
    g.dependency.dependents.retain(|dp_id| *dp_id != dependent);

    let d = model.population.get_mut(&dependent).unwrap();
    d.dependency.guardians.retain(|grd_id| *grd_id != guardian);
}

pub fn set_as_independent(p_id: Id, model: &mut Model) {
    let p = model.population.get_mut(&p_id).unwrap();
    let guardians = mem::take(&mut p.dependency.guardians);
    for g in guardians {
        let g = model.population.get_mut(&g).unwrap();
        g.dependency.dependents.retain(|dp_id| *dp_id != p_id);
    }
}

pub fn set_as_provider_providee(provider_id: Id, providee_id: Id, model: &mut Model) {
    let provider = model.population.get_mut(&provider_id).unwrap();
    debug_assert!(!provider.dependency.providees.contains(&providee_id));
    provider.dependency.providees.push(providee_id);

    let providee = model.population.get_mut(&providee_id).unwrap();
    debug_assert!(providee.dependency.provider.is_none());
    providee.dependency.provider = Some(provider_id);
}

pub fn set_as_self_providing(p_id: Id, model: &mut Model) {
    let p = model.population.get_mut(&p_id).unwrap();
    if let Some(provider_id) = p.dependency.provider.take() {
        let provider = model.population.get_mut(&provider_id).unwrap();
        provider
            .dependency
            .providees
            .retain(|pvd_id| *pvd_id != p_id);
    }
}
