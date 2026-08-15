use crate::{
    agents::agent_modules::work::WorkStatus,
    full_model::{Model, person::Id},
    simulate::tasks_care::care_supply_changed,
};

pub fn change_status(p_id: Id, new_status: WorkStatus, model: &mut Model) {
    let person = model.population.get_mut(&p_id).unwrap();
    person.work.status = new_status;
    care_supply_changed(p_id, model);
}
