use crate::{
    ModelPars,
    full_model::{Model, person::Id},
    simulate::{
        dependencies::process_change_1yr_deps, social::process_change_1yr_social,
        tasks_care::process_change_1yr_task_care,
    },
};

pub fn age_transition(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive_mut(p_id);

    person.maternity.step(pars);
    person.basic.age.add_one_month();

    // FIXME: currently unused
    if let Some(partnership) = &mut person.kinship.partnership {
        partnership.step_one_month();
    }

    if person.basic.has_birthday() {
        process_change_1yr_deps(p_id, model, pars);
        process_change_1yr_social(p_id, model, pars);
        process_change_1yr_task_care(p_id, model, pars);
    }
}
