use std::mem;

use rand::RngExt;

use crate::{
    ModelPars, N_CLASSES,
    agents::{
        agent_modules::basic_info::Gender,
        interactions::{
            dependencies::{set_as_guardian_dependent, set_as_provider_providee},
            family::{has_young_infant, set_as_parent_child},
        },
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    utilities::{Age, Date, calc_rate_bias, try_rand_yearly2monthly},
};

fn is_fertile_woman(person: &Person, pars: &ModelPars) -> bool {
    person.is_female()
        && person.basic.age >= Age::years(pars.birth.min_pregnancy_age)
        && person.basic.age <= Age::years(pars.birth.max_pregnancy_age)
}

fn can_be_pregnant(person: &Person, model: &Model) -> bool {
    !(person.kinship.is_single() || has_young_infant(person, model))
}

fn is_potential_mother(person: &Person, model: &Model, pars: &ModelPars) -> bool {
    is_fertile_woman(person, pars) && can_be_pregnant(person, model)
}

#[derive(Default)]
pub struct BirthCache {
    potential_mothers: Vec<Id>,
    p_potential_mother: Vec<f64>,
    pre51_fert_scaling: f64,
    class_bias: [f64; N_CLASSES],
    n_children_bias: [[f64; 5]; N_CLASSES],
}

pub fn birth_pre_calc(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let mut pot_mothers: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| is_potential_mother(p, model, pars).then_some(p.id()))
        .collect();
    mem::swap(&mut pot_mothers, &mut model.birth_cache.potential_mothers);
    mem::drop(pot_mothers);

    // FIXME: probably don't need to go to 150
    model.birth_cache.p_potential_mother.resize(150, 0.0);
    let mut cbp = vec![0.0; 150];
    for person in model.pop.alives(order) {
        if is_fertile_woman(person, pars) {
            let iy = person.basic.age.year_month().0 as usize;
            cbp[iy] += 1.0;
            if can_be_pregnant(person, model) {
                model.birth_cache.p_potential_mother[iy] += 1.0;
            }
        }
    }

    model.birth_cache.pre51_fert_scaling = 0.0;
    for (i_fert_age, fert51) in model.fert_f_by_age_51.iter().enumerate() {
        let iage = pars.birth.min_pregnancy_age as usize + i_fert_age;
        model.birth_cache.pre51_fert_scaling += model.birth_cache.p_potential_mother[iage] * fert51;
    }
    model.birth_cache.pre51_fert_scaling =
        model.pop.size() as f64 / model.birth_cache.pre51_fert_scaling;

    cbp.into_iter()
        .enumerate()
        .filter(|p| p.1 > 0.0)
        .for_each(|(iy, n)| model.birth_cache.p_potential_mother[iy] /= n);

    let mut n_per_class = [0.0; N_CLASSES];
    model
        .birth_cache
        .potential_mothers
        .iter()
        .map(|&id| model.pop.alive(id))
        .for_each(|p| n_per_class[p.class.rank_idx()] += 1.0);

    let npotmoms = model.birth_cache.potential_mothers.len().max(1) as f64;
    let pcpm = n_per_class.map(|pc| pc / npotmoms);
    model.birth_cache.class_bias = calc_rate_bias(|i| pcpm[i], pars.birth.fertility_bias);

    let mut pncpmc = [[0.0; 5]; N_CLASSES];
    model
        .birth_cache
        .potential_mothers
        .iter()
        .map(|&id| model.pop.alive(id))
        .for_each(|p| pncpmc[p.class.rank_idx()][p.kinship.n_children().min(4)] += 1.0);
    for ic in 0..N_CLASSES {
        for nc in 0..5 {
            pncpmc[ic][nc] /= n_per_class[ic];
        }
        model.birth_cache.n_children_bias[ic] =
            calc_rate_bias(|nc| pncpmc[ic][nc], pars.birth.prev_child_fertility_bias);
    }
}

fn compute_birth_prob(woman: &Person, model: &Model, pars: &ModelPars, date: Date) -> f64 {
    let (year, _) = date.year_month();

    let rank = if woman.work.is_student() {
        woman.class.parent_rank
    } else {
        woman.class.rank
    };

    let age_years = woman.basic.age.year_month().0;
    let ifert_age = (age_years - pars.birth.min_pregnancy_age) as usize;

    // FIXME: abstract this in a fertility object
    let raw_rate = if year < 1951 {
        // pre 51 is the number of children per UK resident and year,
        // birth rate is then rescaled to match the distribution of 1951
        model.fert_pre51.in_year(year)
            * model.fert_f_by_age_51[ifert_age]
            * model.birth_cache.pre51_fert_scaling
    } else {
        // fertility rates are stored as P(pregnant) per year and age
        model.fert_post51.rate_for(ifert_age, year)
            / model.birth_cache.p_potential_mother[age_years as usize]
    };

    let irank = rank.index();
    // apply class bias
    let mut birth_prob = raw_rate * model.birth_cache.class_bias[irank];

    // apply bias due to number of previous children
    let nchildren = woman.kinship.n_children().min(4);
    birth_prob *= model.birth_cache.n_children_bias[irank][nchildren];

    birth_prob.clamp(0.0, 1.0)
}

fn effects_of_maternity(woman_id: Id, model: &mut Model) {
    let woman = model.pop.alive_mut(woman_id);

    woman.maternity.start();

    // FIXME: work tasks need to be suspended while on maternity leave and then accepted again when
    // leave ends.
    woman.work.working_hours = 0;
    woman.work.income = 0.0;
    woman.work.available_working_hours = 0;

    // TODO: not necessarily true in many cases
    if woman.dependency.provider.is_none()
        && let Some(partner_id) = woman.kinship.partner()
    {
        set_as_provider_providee(partner_id, woman_id, model);
    }
}

pub fn select_birth(person: &Person, model: &Model, pars: &ModelPars) -> bool {
    is_potential_mother(person, model, pars)
}

pub fn birth(woman_id: Id, date: Date, model: &mut Model, pars: &ModelPars) {
    let woman = model.pop.alive(woman_id);
    let birth_prob = compute_birth_prob(woman, model, pars, date);

    // FIXME: all these should be true by design
    assert!(woman.is_female());
    assert!(!has_young_infant(woman, model));
    assert!(!woman.kinship.is_single());
    assert!(woman.basic.age >= Age::years(pars.birth.min_pregnancy_age));
    assert!(woman.basic.age <= Age::years(pars.birth.max_pregnancy_age));

    if try_rand_yearly2monthly(birth_prob, &mut model.rng) {
        let gender = if model.rng.random_bool(0.5) {
            Gender::Male
        } else {
            Gender::Female
        };
        let house = woman.house;
        let baby = Person::baby(gender, house);
        let baby_id = baby.id();
        let maybe_partner = woman.kinship.partner();
        model
            .houses
            .get_mut(&house)
            .unwrap()
            .basic
            .add_occupant(baby_id);

        // Note: the baby is not iterated over until the next loop.
        model.pop.insert(baby);

        set_as_parent_child(baby_id, woman_id, model);
        if let Some(partner_id) = maybe_partner {
            set_as_parent_child(baby_id, partner_id, model);
        }

        // this goes first, so that we know material circumstances
        effects_of_maternity(woman_id, model);

        set_as_guardian_dependent(woman_id, baby_id, model);
        if let Some(partner_id) = maybe_partner {
            set_as_guardian_dependent(partner_id, baby_id, model);
        }

        set_as_provider_providee(woman_id, baby_id, model);
    }
}
