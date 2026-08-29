use crate::{
    agents::agent_modules::{tasks::empty_todo, work::WorkStatus},
    full_model::{Model, person::Id},
};

pub fn change_status(p_id: Id, new_status: WorkStatus, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    person.work.status = new_status;
    empty_todo(p_id, model);
}
