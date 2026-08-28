use rand::{Rng, RngExt};
use rand_distr::{LogNormal, Normal};

use crate::{
    ModelPars,
    full_model::{Model, person::Person},
    population::PopIterOrder,
};

/// Set initial and final wage depending on social class.
pub fn set_wage_progression<R: Rng>(person: &mut Person, rng: &mut R, pars: &ModelPars) {
    let r = person.class.rank_idx();

    let dki_distr = Normal::new(0.0, pars.work.wage_var).unwrap();
    let dki = rng.sample(dki_distr);
    person.work.initial_wage = pars.work.income_initial_levels[r] * dki.exp();

    let fw_dist = LogNormal::new(
        pars.work.final_income_mu[r],
        pars.work.final_income_sigma[r],
    )
    .unwrap();
    person.work.final_wage = rng.sample(fw_dist);
}

/// Set agent wealth dependent on cumulative income.
pub fn assign_wealth_by_inc_percentile(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let mut pop: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| {
            (p.work.cumulative_income > 0.0).then_some((p.id(), p.work.cumulative_income))
        })
        .collect();
    pop.sort_by(|p1, p2| p1.1.partial_cmp(&p2.1).unwrap());

    let pop_length = pop.len();
    let dk_distr = Normal::new(0.0, pars.work.wage_var).unwrap();
    let w_ptile = model.wealth_percentiles.percentiles();

    for (i, (id, _)) in pop.into_iter().enumerate() {
        let percentile = (100 * i) / pop_length;
        let dk = model.rng.sample(dk_distr);
        let agent = model.pop.alive_mut(id);
        agent.work.wealth = w_ptile[percentile] * dk.exp();
    }
}

// FIXME: reduce dupplication with `assign_wealth_by_inc_percentile`
/// Set house holds wealth dependent on cumulative income.
pub fn assign_wealth_by_inc_percentile_hh(model: &mut Model, pars: &ModelPars) {
    let mut pop: Vec<_> = model.houses.values_mut().collect();
    pop.sort_by(|p1, p2| {
        let inc1 = p1.income.cumulative_income;
        let inc2 = p2.income.cumulative_income;
        inc1.partial_cmp(&inc2).unwrap()
    });

    let pop_length = pop.len();
    let dk_distr = Normal::new(0.0, pars.work.wage_var).unwrap();
    let w_ptile = model.wealth_percentiles.percentiles();

    for (i, agent) in pop.into_iter().enumerate() {
        let percentile = (100 * i) / pop_length;
        let dk = model.rng.sample(dk_distr);
        agent.income.wealth = w_ptile[percentile] * dk.exp();
    }
}

/// Calculate current wage dependent on initial and final wage and work experience.
pub fn compute_wage<R: Rng>(person: &Person, rng: &mut R, pars: &ModelPars) -> f64 {
    let fw = person.work.final_wage;
    let iw = person.work.initial_wage;
    let r = person.class.rank_idx();

    let exponent = -pars.work.income_growth_rate[r] * person.work.work_experience;
    let wage = fw * (iw / fw).powf(exponent.exp());

    let dk_distr = Normal::new(0.0, pars.work.wage_var).unwrap();
    let dk = rng.sample(dk_distr);

    wage * dk.exp()
}
