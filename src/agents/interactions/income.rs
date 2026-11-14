use crate::{
    full_model::{Model, person::Id},
    per_time::Weekly,
};

// FIXME: should these take a IdHouse directly instead?
fn hh_income(p_id: Id, model: &Model) -> (Weekly, usize) {
    let person = model.pop.alive(p_id);
    let house = person.house(model);
    let occupants = house.basic.occupants();
    let income = occupants
        .iter()
        .map(|&id| model.pop.alive(id).work.income)
        .sum();
    (income, occupants.len())
}

pub fn household_income_per_capita(p_id: Id, model: &Model) -> Weekly {
    let (income, noccupants) = hh_income(p_id, model);
    income / noccupants as f64
}
