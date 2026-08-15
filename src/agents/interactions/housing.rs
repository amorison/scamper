use crate::full_model::{
    Model,
    house::IdHouse,
    person::{Id, Person},
};

/// Associate a house to a person, remove person from previous house.
pub fn move_to_house(p_id: Id, h_id: IdHouse, model: &mut Model) {
    let person = model.population.get_mut(&p_id).unwrap();
    let prev_house = person.house;

    if prev_house == h_id {
        return;
    }

    person.house = h_id;

    let prev_house = model.houses.get_mut(&prev_house).unwrap();
    prev_house.basic.rm_occupant(p_id);

    let house = model.houses.get_mut(&h_id).unwrap();
    house.basic.add_occupant(p_id);
}

pub fn living_together(p1: &Person, p2: &Person) -> bool {
    p1.house == p2.house
}
