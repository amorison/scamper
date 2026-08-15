use rand::seq::IndexedRandom;

use crate::{
    agents::{interactions::housing::move_to_house, towns::IdTown},
    common::world::{empty_houses, empty_houses_in_town},
    full_model::{Model, house::IdHouse, person::Id},
};

fn find_empty_house_in_town(t_id: IdTown, model: &mut Model) -> Option<IdHouse> {
    let houses = empty_houses_in_town(t_id, model);
    houses.choose(&mut model.rng).copied()
}

fn find_empty_house_in_or_adjacent_town(t_id: IdTown, model: &mut Model) -> Option<IdHouse> {
    let town = model.towns.get(&t_id).unwrap();
    let mut empty_houses = empty_houses_in_town(t_id, model);
    for &t_id in &town.adjacent {
        empty_houses.extend(empty_houses_in_town(t_id, model));
    }
    empty_houses.choose(&mut model.rng).copied()
}

// cache empty houses?
fn find_empty_house_anywhere(model: &mut Model) -> Option<IdHouse> {
    let houses = empty_houses(model);
    houses.choose(&mut model.rng).copied()
}

pub fn move_people_to_house(ids: &[Id], h_id: IdHouse, model: &mut Model) {
    // TODO: (comment in Julia)
    // - yearInTown (used in relocation cost)
    // - movedThisYear
    for &p_id in ids {
        move_to_house(p_id, h_id, model);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proximity {
    Here,
    Near,
    Far,
}

pub fn move_people_to_empty_house(
    ids: &[Id],
    proximity: Proximity,
    model: &mut Model,
) -> Option<IdHouse> {
    // first person determines search area
    let person0 = model.population.get(&ids[0]).unwrap();
    let t_id = person0.house(model).basic.town();

    let mut new_house = None;

    if proximity == Proximity::Here {
        new_house = find_empty_house_in_town(t_id, model);
    }
    if proximity == Proximity::Near || new_house.is_none() {
        new_house = find_empty_house_in_or_adjacent_town(t_id, model);
    }
    if proximity == Proximity::Far || new_house.is_none() {
        new_house = find_empty_house_anywhere(model);
    }

    if let Some(h_id) = new_house {
        move_people_to_house(ids, h_id, model);
    }

    new_house
}
