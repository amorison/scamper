use rand_distr::{Distribution, Geometric};

use crate::{
    ModelPars, N_CARE_LEVELS, N_CLASSES,
    agents::{
        agent_modules::{basic_info::Gender, tasks::empty_todo},
        tasks::TaskKind,
    },
    full_model::{Model, person::Id},
    simulate::tasks_care::care_need_changed,
    utilities::{calc_rate_bias, try_rand_yearly2monthly},
};

#[derive(Default)]
pub struct SocialCareCache {
    class_bias: [f64; N_CLASSES],
}

pub fn social_care_pre_calc(model: &mut Model, pars: &ModelPars) {
    model.social_care_cache.class_bias =
        calc_rate_bias(model.social_cache.social_class_shares, pars.care.care_bias);
}

/// Adjust social care need.
pub fn social_care_transition(p_id: Id, model: &mut Model, pars: &ModelPars) -> bool {
    let person = model.pop.alive_mut(p_id);
    let unmet_care = person.task.open_tasks.ntasks(TaskKind::SocialCare);
    person.care.register_unmet_need(unmet_care as f64, pars);
    let scaling = match person.basic.gender {
        Gender::Female => pars.care.female_age_care_scaling,
        Gender::Male => pars.care.male_age_care_scaling,
    };

    let age_care_prob = (person.basic.age.years_f64() / scaling).exp() * pars.care.person_care_prob;
    let mut base_prob = pars.care.base_care_prob + age_care_prob;

    let irank = person.class.rank_idx();
    base_prob *= model.social_care_cache.class_bias[irank];
    let unmet_need_factor = pars.care.unmet_need_exp * person.care.current_unmet_need();
    base_prob *= unmet_need_factor;

    base_prob = base_prob.clamp(0.0, 1.0);

    if !try_rand_yearly2monthly(base_prob, &mut model.rng) {
        return false;
    }

    let transition_rate = (1.0
        - pars.care.care_transition_rate * model.social_care_cache.class_bias[irank])
        / unmet_need_factor;
    let transition_rate = transition_rate.clamp(0.0, 1.0);

    let distr = Geometric::new(transition_rate).unwrap();
    let care_need = person.care.need_level + 1 + distr.sample(&mut model.rng) as u32;
    person.care.need_level = care_need.min((N_CARE_LEVELS - 1) as u32);

    care_need_changed(p_id, model, pars);
    empty_todo(p_id, model);
    // FIXME: need to recompute job tasks accordingly?

    true
}
