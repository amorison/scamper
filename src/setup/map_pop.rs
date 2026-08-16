use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::full_model::{Model, person::build::PersonAwaitingHouse};

pub fn assign_couples_to_houses(mut population: Vec<PersonAwaitingHouse>, model: &mut Model) {
    let with_partner_or_deps: Vec<_> = population
        .extract_if(0..population.len(), |p| {
            p.kinship.partner().is_some() || p.dependency.has_dependents()
        })
        .collect();
    let rest = population;

    let mut houses: Vec<_> = model.houses.keys().copied().collect();
    houses.shuffle(&mut model.rng);

    // probably a pessimistic capacity
    let mut have_house = HashMap::with_capacity(with_partner_or_deps.len());

    for person in with_partner_or_deps.into_iter().chain(rest.into_iter()) {
        let p_id = person.id();

        let h_id = if let Some(h_id) = have_house.remove(&p_id) {
            h_id
        } else {
            houses
                .pop()
                .expect("ran out of houses when lodging initial population")
        };

        let person = person.with_house(h_id);

        // Make sure partners and children/parents are together. Note: because we handle all agents
        // with a partner or dependents first, there is no need to attribute explicitly the same
        // house to guardians (and in turn the dependents of the guardians). These conditions aren't
        // hit anymore when we're in the "rest".
        if let Some(partner_id) = person.kinship.partner() {
            have_house.insert(partner_id, h_id);
        }
        for &child_id in &person.dependency.dependents {
            have_house.insert(child_id, h_id);
        }

        let house = model.houses.get_mut(&h_id).unwrap();
        house.basic.add_occupant(p_id);

        model.pop.insert(person);
    }
}
