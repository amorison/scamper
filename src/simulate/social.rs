use rand::RngExt;
use rand_distr::{Distribution, Normal};

use crate::{
    ModelPars, N_CLASSES,
    agents::{
        agent_modules::work::{WorkStatus, lose_job},
        interactions::{dependencies::set_as_self_providing, income::household_income_per_capita},
    },
    common::{income::set_wage_progression, social::change_status},
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    utilities::Age,
};

fn start_retirement(p_id: Id, model: &mut Model, pars: &ModelPars) {
    lose_job(p_id, model);

    let person = model.pop.alive_mut(p_id);
    let share_working_time = person.work.working_periods / pars.work.min_contribution_period as f64;

    let dk_dist = Normal::new(0.0, pars.work.wage_var).unwrap();
    let dk = dk_dist.sample(&mut model.rng);
    person.work.pension = person.work.last_income * share_working_time * dk.exp();
}

pub fn process_change_1yr_social(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive_mut(p_id);
    let age = person.basic.age;
    if age == Age::years(pars.work.age_teenagers) {
        change_status(p_id, WorkStatus::Teenager, model);
    } else if age == Age::years(pars.work.age_adulthood) {
        // all agents first become students, start working in social transition
        change_status(p_id, WorkStatus::Student, model);
    } else if age == Age::years(pars.work.age_retirement) {
        start_retirement(p_id, model, pars);
        change_status(p_id, WorkStatus::Retired, model);
    }
}

#[derive(Default)]
pub struct SocialCache {
    pub social_class_shares: [f64; N_CLASSES],
}

pub fn social_pre_calc(model: &mut Model, order: &PopIterOrder) {
    for person in model.pop.alives(order) {
        model.social_cache.social_class_shares[person.class.rank_idx()] += 1.0;
    }

    let tot: f64 = model.social_cache.social_class_shares.iter().sum();
    model
        .social_cache
        .social_class_shares
        .iter_mut()
        .for_each(|val| *val /= tot);
}

pub fn student_start_working(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive_mut(p_id);
    person.class.enter_workforce();
    set_wage_progression(person, &mut model.rng, pars);
    set_as_self_providing(p_id, model);
    change_status(p_id, WorkStatus::Unemployed, model);
}

// FIXME: use where appropriate
/// Age at which a person can decide to stop studying and start working.
pub fn start_working_age(person: &Person, pars: &ModelPars) -> Age {
    Age::years(pars.work.start_working_age[person.class.rank_idx()])
}

pub fn select_social_transition(person: &Person, pars: &ModelPars) -> bool {
    // check once a year
    person.basic.has_birthday()
        && person.basic.age == start_working_age(person, pars)
        && person.work.is_student()
}

/// Decide whether agent goes on to study or starts working.
pub fn social_transition(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive(p_id);
    let prob_study = if person.class.has_max_education() {
        0.0
    } else {
        start_study_prob(person, model, pars)
    };

    if model.rng.random_bool(prob_study) {
        let person = model.pop.alive_mut(p_id);
        person.class.study();
    } else {
        student_start_working(p_id, model, pars);
    }
}

/// Probability to start studying instead of working.
fn start_study_prob(person: &Person, model: &Model, pars: &ModelPars) -> f64 {
    if person.kinship.father.is_none() && person.kinship.mother.is_none() {
        return 0.0;
    }

    if person.dependency.provider.is_none() {
        return 0.0;
    }

    let per_capital_disposable_income = household_income_per_capita(person.id(), model);
    if per_capital_disposable_income <= 0.0 {
        return 0.0;
    }

    let irank = person.class.rank_idx();
    let forgone_salary =
        pars.work.income_initial_levels[irank] * pars.work.weekly_hours[person.care.index()] as f64;
    let rel_cost = forgone_salary / per_capital_disposable_income;
    let income_effect = (pars.work.constant_income + 1.0)
        / ((pars.work.edu_wage_sensitivity * rel_cost).exp() + pars.work.constant_income);

    let education_effect = person.class.edu_gap_factor(pars);

    let care_work = (person.care.social_work + person.care.child_work) as f64;
    let care_effect = 1.0 / (pars.work.care_education * care_work).exp();

    let p_study = income_effect * education_effect * care_effect;

    p_study.clamp(0.0, 1.0)
}
