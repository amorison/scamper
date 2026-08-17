use rand::RngExt;

use crate::{
    ModelPars, N_AGE_BANDS, N_CLASSES,
    agents::agent_modules::work::{WorkStatus, lose_job},
    common::{
        income::compute_wage,
        job_market::{assign_jobs, calc_age_class_shares, compute_ur_by_class_age},
        social::change_status,
    },
    full_model::{
        Model,
        person::{Id, Person, weekly_todo_tally},
    },
    population::PopIterOrder,
    utilities::Date,
};

#[derive(Default)]
pub struct JobCache {
    urates: [[f64; N_AGE_BANDS]; N_CLASSES],
}

pub fn job_pre_calc(date: Date, model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let shares = calc_age_class_shares(model, order);
    let unemployment_rate = model.unemployment_series.rate(date);
    model.job_cache.urates = compute_ur_by_class_age(unemployment_rate, shares, pars);

    // FIXME: is that the right place? Ideally we would have stateless diagnostics...
    for p_id in order.ids() {
        let tally = weekly_todo_tally(p_id, model);
        let person = model.pop.alive_mut(p_id);
        person.work.available_working_hours = tally.work;
    }
}

fn calc_fire_probability(p_id: Id, model: &Model, pars: &ModelPars) -> f64 {
    // TODO: (Julia comment) From old version, keep?
    let person = model.pop.alive(p_id);
    // FIXME: check unit of job_tenure
    if person.work.job_tenure < 1 {
        return 0.0;
    }

    let ur = model.job_cache.urates[person.class.rank_idx()][person.basic.age.band()];
    1.0 - 1.0 / (pars.work.hire_rate / (1.0 / ur - 1.0)).exp()
}

pub fn select_employed(person: &Person) -> bool {
    person.work.is_worker()
}

pub fn employed_transition(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let prob_fired = calc_fire_probability(p_id, model, pars);

    if model.rng.random_bool(prob_fired) {
        lose_job(p_id, model);
        change_status(p_id, WorkStatus::Unemployed, model);
    } else {
        let person = model.pop.alive_mut(p_id);
        person.work.job_tenure += 1;
        if person.work.working_hours > 0 {
            person.work.working_periods +=
                person.work.available_working_hours as f64 / person.work.working_hours as f64;
        }
        person.work.work_experience +=
            person.work.available_working_hours as f64 / pars.work.weekly_hours[0] as f64;
        person.work.wage = compute_wage(person, &mut model.rng, pars)
    }
}

fn calc_hire_probability(pars: &ModelPars) -> f64 {
    1.0 - 1.0 / pars.work.hire_rate.exp()
}

pub fn select_unemployed(person: &Person) -> bool {
    person.work.is_unemployed()
}

pub fn unemployed_transition(p_id: Id, date: Date, model: &mut Model, pars: &ModelPars) {
    let prob_hired = calc_hire_probability(pars);

    if model.rng.random_bool(prob_hired) {
        // TODO: adapt
        assign_jobs(&[p_id], date, pars, model);
    }
}
