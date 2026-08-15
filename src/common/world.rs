use crate::{
    agents::towns::IdTown,
    full_model::{
        Model,
        house::{House, IdHouse},
    },
};

pub fn empty_houses(model: &Model) -> Vec<IdHouse> {
    model
        .houses
        .values()
        .filter(|h| h.basic.is_empty())
        .map(House::id)
        .collect()
}

pub fn empty_houses_in_town(t_id: IdTown, model: &Model) -> Vec<IdHouse> {
    let town = model.towns.get(&t_id).unwrap();
    town.houses
        .iter()
        .copied()
        .filter(|h_id| {
            let house = model.houses.get(h_id).unwrap();
            house.basic.is_empty()
        })
        .collect()
}
