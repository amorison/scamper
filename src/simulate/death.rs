use crate::{
    MAX_AGE, ModelPars, N_CARE_LEVELS, N_CLASSES,
    agents::{
        agent_modules::{
            basic_info::Gender,
            work::{WorkStatus, lose_job},
        },
        interactions::family::resolve_partnership,
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    simulate::{dependencies::process_death_deps, tasks_care::process_death_task_care},
    utilities::{Age, Date, sum_class_bias, try_rand_yearly2monthly},
};

fn death_probability(base_rate: f64, person: &Person, model: &Model, pars: &ModelPars) -> f64 {
    let irank =
        if person.work.status == WorkStatus::Child || person.work.status == WorkStatus::Student {
            person.class.parent_rank
        } else {
            person.class.rank
        }
        .index();

    let (mortality_bias, sum_bias) = match person.basic.gender {
        Gender::Female => (
            pars.population.male_mortality_bias,
            model.death_cache.class_bias_m,
        ),
        Gender::Male => (
            pars.population.female_mortality_bias,
            model.death_cache.class_bias_f,
        ),
    };

    let death_prob = if sum_bias > 0.0 {
        base_rate * mortality_bias.powi(irank as i32) / sum_bias
    } else {
        base_rate
    };

    let b = model.death_cache.care_bias[irank];
    if b > 0.0 {
        let higher_need_rate = death_prob / b;
        let exp = N_CARE_LEVELS as i32 - 1 - person.care.need_level as i32;
        higher_need_rate * pars.care.care_need_bias.powi(exp)
    } else {
        death_prob
    }
    .clamp(0.0, 1.0)
}

fn set_dead(p_id: Id, model: &mut Model) {
    lose_job(p_id, model);

    let person = model.pop.alive_mut(p_id);

    let house = model.houses.get_mut(&person.house).unwrap();
    house.basic.rm_occupant(p_id);

    if let Some(partner_id) = person.kinship.partner() {
        resolve_partnership(partner_id, p_id, model);
    }

    process_death_task_care(p_id, model);
    process_death_deps(p_id, model);

    model.carehomes.dead(p_id);
    model.pop.mark_as_dead(p_id);
}

#[derive(Default)]
pub struct DeathCache {
    avg_die_prob_m: f64,
    avg_die_prob_f: f64,
    class_bias_m: f64,
    class_bias_f: f64,
    care_bias: [f64; N_CLASSES],
}

pub fn death_pre_calc(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let mut care_need_shares = [[0.0; N_CARE_LEVELS]; N_CLASSES];
    let mut s_m = 0.0;
    let mut s_f = 0.0;
    let mut n_m = 0.0;
    let mut n_f = 0.0;

    for person in model.pop.alives(order) {
        let die_prob = age_die_prob(person.basic.age, person.basic.gender, pars);
        match person.basic.gender {
            Gender::Female => {
                s_f += die_prob;
                n_f += 1.0;
            }
            Gender::Male => {
                s_m += die_prob;
                n_m += 1.0;
            }
        }
        care_need_shares[person.class.rank_idx()][person.care.index()] += 1.0;
    }

    model.death_cache.avg_die_prob_m = s_m / n_m;
    model.death_cache.avg_die_prob_f = s_f / n_f;

    model.death_cache.class_bias_m = sum_class_bias::<_, N_CLASSES>(
        |i| model.social_cache.social_class_shares[i],
        pars.population.male_mortality_bias,
    );
    model.death_cache.class_bias_f = sum_class_bias::<_, N_CLASSES>(
        |i| model.social_cache.social_class_shares[i],
        pars.population.female_mortality_bias,
    );

    for ic in 0..N_CLASSES {
        let s: f64 = care_need_shares[ic].iter().sum();
        care_need_shares[ic] = care_need_shares[ic].map(|cns| cns / s);
    }

    for ic in 0..N_CLASSES {
        model.death_cache.care_bias[ic] = care_need_shares[ic]
            .iter()
            .enumerate()
            .map(|(i_cn, cns)| {
                let exp = N_CARE_LEVELS as i32 - 1 - i_cn as i32;
                cns * pars.care.care_need_bias.powi(exp)
            })
            .sum()
    }
}

fn age_die_prob(age: Age, gender: Gender, pars: &ModelPars) -> f64 {
    let age = age.year_month().0 as f64;
    let (age_scaling, age_die_prob) = match gender {
        Gender::Female => (
            pars.population.female_age_scaling,
            pars.population.female_age_die_prob,
        ),
        Gender::Male => (
            pars.population.male_age_scaling,
            pars.population.male_age_die_prob,
        ),
    };
    pars.population.base_die_prob + (age / age_scaling).exp() * age_die_prob
}

fn death_due(p_id: Id, date: Date, model: &mut Model, pars: &ModelPars) -> bool {
    let person = model.pop.alive(p_id);

    if person.basic.age >= MAX_AGE {
        return true;
    }

    let (year, _) = date.year_month();

    let raw_rate = if year < 1950 {
        // made-up probabilities
        if person.basic.age < Age::years(1) {
            model.pre51_deaths.infant_rate(date)
        } else {
            let avg_prob = match person.basic.gender {
                Gender::Female => model.death_cache.avg_die_prob_f,
                Gender::Male => model.death_cache.avg_die_prob_m,
            };
            model.pre51_deaths.rate(date)
                * age_die_prob(person.basic.age, person.basic.gender, pars)
                / avg_prob
        }
    } else {
        match person.basic.gender {
            Gender::Female => model.female_deaths.rate(person.basic.age, date),
            Gender::Male => model.male_deaths.rate(person.basic.age, date),
        }
    };

    let death_prob = death_probability(raw_rate, person, model, pars);

    try_rand_yearly2monthly(death_prob, &mut model.rng)
}

pub fn death(p_id: Id, date: Date, model: &mut Model, pars: &ModelPars) {
    if death_due(p_id, date, model, pars) {
        set_dead(p_id, model);
    }
}
