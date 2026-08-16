use rand_distr::{Distribution, Geometric};

use crate::{
    ModelPars, N_CARE_LEVELS, N_CLASSES,
    agents::agent_modules::{basic_info::Gender, work::WorkStatus},
    full_model::{Model, person::Id},
    simulate::tasks_care::{care_need_changed, care_supply_changed},
    utilities::{calc_rate_bias, try_rand_yearly2monthly},
};

#[derive(Default)]
pub struct SocialCareCache {
    class_bias: [f64; N_CLASSES],
}

pub fn social_care_pre_calc(model: &mut Model, pars: &ModelPars) {
    model.social_care_cache.class_bias = calc_rate_bias(
        |i| model.social_cache.social_class_shares[i],
        pars.care.care_bias,
    );
}

/// Adjust social care need.
pub fn social_care_transition(p_id: Id, model: &mut Model, pars: &ModelPars) -> bool {
    let person = model.pop.alive_mut(p_id);
    let scaling = match person.basic.gender {
        Gender::Female => pars.care.female_age_care_scaling,
        Gender::Male => pars.care.male_age_care_scaling,
    };

    let age_care_prob = (person.basic.age.years_f64() / scaling).exp() * pars.care.person_care_prob;
    let mut base_prob = pars.care.base_care_prob + age_care_prob;

    let class = match person.work.status {
        // FIXME: julia missed teenager here
        WorkStatus::Child | WorkStatus::Teenager | WorkStatus::Student => person.class.parent_rank,
        WorkStatus::FixedShiftEmployed
        | WorkStatus::FlexibleShiftEmployed
        | WorkStatus::Retired
        | WorkStatus::Unemployed => person.class.rank,
    };

    let irank = class.index();
    base_prob *= model.social_care_cache.class_bias[irank];
    base_prob = base_prob.clamp(0.0, 1.0);

    if !try_rand_yearly2monthly(base_prob, &mut model.rng) {
        return false;
    }

    let transition_rate =
        pars.care.care_transition_rate * model.social_care_cache.class_bias[irank];

    let distr = Geometric::new(1.0 - transition_rate).unwrap();
    let care_need = person.care.need_level + 1 + distr.sample(&mut model.rng) as u32;
    person.care.need_level = care_need.min((N_CARE_LEVELS - 1) as u32);

    care_need_changed(p_id, model, pars);
    care_supply_changed(p_id, model);

    true
}
