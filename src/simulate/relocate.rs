use rand::RngExt;

use crate::{
    ModelPars,
    agents::interactions::{
        dependencies::{can_live_alone, lives_in_shared_house},
        housing::living_together,
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    simulate::move_house::{Proximity, move_people_to_empty_house},
};

pub fn select_relocate(person: &Person, model: &Model) -> bool {
    can_live_alone(person)
        && person.work.is_worker()
        && person.kinship.is_single()
        && lives_in_shared_house(person.id(), model)
}

pub fn relocate(p_id: Id, model: &mut Model, pars: &ModelPars) -> bool {
    if model.rng.random_bool(pars.work.move_out_prob) {
        let mut people_to_move = vec![p_id];
        let person = model.pop.alive(p_id);
        for &dep_id in &person.dependency.dependents {
            let dep = model.pop.alive(dep_id);
            if living_together(person, dep) {
                people_to_move.push(dep_id);
            }
        }

        let prox = if model.rng.random_bool(0.5) {
            Proximity::Near
        } else {
            Proximity::Far
        };
        move_people_to_empty_house(&people_to_move, prox, model);

        true
    } else {
        false
    }
}
