use std::cmp;

use identity_hash::IntMap;
use rand::{Rng, RngExt, seq::IndexedRandom};
use rand_distr::uniform::SampleRange;

use crate::{
    ModelPars, N_CLASSES,
    agents::agent_modules::{
        basic_info::Gender, class::Rank, kinship::Partnership, work::WorkStatus,
    },
    common::{
        income::{assign_wealth_by_inc_percentile_hh, compute_wage, set_wage_progression},
        job_market::assign_jobs,
        tasks_care::{init_care_tasks, social_care_demand_per_day},
    },
    full_model::{
        Model,
        data::AgePyramid,
        person::{Id, Person, build::PersonAwaitingHouse},
    },
    population::PopIterOrder,
    utilities::{Age, Date, int_map_with_cap},
};

// FIXME: this is inefficient, sampling for the entire population in one go would be better
fn rand_age<R: Rng>(pyramid: &AgePyramid, gender: Gender, rng: &mut R) -> Age {
    let data = pyramid.of_gender(gender);
    let indices: Vec<_> = (0..data.len()).collect();
    let idx = *indices.choose_weighted(rng, |&i| data[i]).unwrap() as u32;
    // bins of 5 years
    let min_months = idx * 5 * 12;
    let max_months = (idx + 1) * 5 * 12;
    let months = (min_months..max_months).sample_single(rng).unwrap();
    Age::months(months)
}

// FIXME: rm duplication with `agents/interations/family`
pub fn set_as_parent_child(
    child_id: Id,
    parent_id: Id,
    population: &mut IntMap<Id, PersonAwaitingHouse>,
) {
    let parent = population.get_mut(&parent_id).unwrap();
    let age_parent = parent.basic.age;
    let gender_parent = parent.basic.gender;
    parent.kinship.add_child(child_id);

    let child = population.get_mut(&child_id).unwrap();
    debug_assert!(child.basic.age < age_parent);
    match gender_parent {
        Gender::Female => {
            assert!(child.kinship.mother.is_none());
            child.kinship.mother = Some(parent_id);
        }
        Gender::Male => {
            assert!(child.kinship.father.is_none());
            child.kinship.father = Some(parent_id);
        }
    }
}

// FIXME: rm duplication with `agents/interations/dependencies`
pub fn set_as_guardian_dependent(
    guardian: Id,
    dependent: Id,
    population: &mut IntMap<Id, PersonAwaitingHouse>,
) {
    let g = population.get_mut(&guardian).unwrap();
    g.dependency.dependents.push(dependent);
    let g_class_rank = g.class.rank;

    let d = population.get_mut(&dependent).unwrap();
    d.dependency.guardians.push(guardian);
    d.class.parent_rank = cmp::max(d.class.parent_rank, g_class_rank);
}

// FIXME: rm duplication with `agents/interations/dependencies`
pub fn set_as_provider_providee(
    provider_id: Id,
    providee_id: Id,
    population: &mut IntMap<Id, PersonAwaitingHouse>,
) {
    let provider = population.get_mut(&provider_id).unwrap();
    debug_assert!(!provider.dependency.providees.contains(&providee_id));
    provider.dependency.providees.push(providee_id);

    let providee = population.get_mut(&providee_id).unwrap();
    debug_assert!(providee.dependency.provider.is_none());
    providee.dependency.provider = Some(provider_id);
}

pub fn create_pyramid_population<R: Rng>(
    pars: &ModelPars,
    order: &mut PopIterOrder,
    rng: &mut R,
) -> Vec<PersonAwaitingHouse> {
    // FIXME: double check order is reproducible, including in how the returned
    // vector is used
    let pyramid = AgePyramid::read_from(&pars.data_files.ini_age);

    let npop = pars.population.init_size as usize;
    let mut men = Vec::with_capacity(npop / 2);
    let mut women = Vec::with_capacity(npop / 2);
    let mut population = int_map_with_cap(npop);

    // create population distributed according to pyramid
    for _ in 0..pars.population.init_size {
        let gender = if rng.random_bool(pars.population.init_male_prop) {
            Gender::Male
        } else {
            Gender::Female
        };
        let age = rand_age(&pyramid, gender, rng);
        let person = PersonAwaitingHouse::new(gender, age);
        order.insert(person.id());
        if age < Age::years(18) {
            population.insert(person.id(), person);
        } else {
            // setting the potential partners apart
            match gender {
                Gender::Female => women.push(person),
                Gender::Male => men.push(person),
            }
        }
    }

    // assign partners
    let n_couples = (pars.population.start_prob_married * men.len() as f64).floor() as usize;
    let mut n_coupled = 0;
    while n_coupled < n_couples {
        let mut man = men
            .pop()
            .expect("trying to form more couples than there are suitable men");
        let w_ages = man.basic.age.range_around_years(-5, 2);
        if let Some(w_idx) = women.iter().position(|w| w_ages.contains(w.basic.age)) {
            let mut woman = women.swap_remove(w_idx);
            man.kinship.partnership = Some(Partnership::new(woman.id()));
            woman.kinship.partnership = Some(Partnership::new(man.id()));
            population.insert(woman.id(), woman);
            n_coupled += 1;
        }
        population.insert(man.id(), man);
    }

    population.extend(men.into_iter().map(|p| (p.id(), p)));
    population.extend(women.into_iter().map(|p| (p.id(), p)));

    // assign parents
    let mut potential_mothers = Vec::with_capacity(2 * population.len() / 3);
    // Can be simplified with `Iterator::collect_into` once it is stabilised.
    let potential_mothers_iter = population.iter().filter_map(|(id, w)| {
        let age = w.basic.age;
        (matches!(w.basic.gender, Gender::Female) && age >= Age::years(18)).then_some((*id, age))
    });
    potential_mothers.extend(potential_mothers_iter);

    let mut pm_in_age_range = Vec::with_capacity(potential_mothers.len());

    for p_id in order.ids() {
        let person = population.get_mut(&p_id).unwrap();
        let child_age = person.basic.age;
        let age_years = child_age.year_month().0;
        // adults remain orphans with a certain likelihood
        if age_years >= 18 && rng.random_bool(pars.population.start_prob_orphan * age_years as f64)
        {
            continue;
        }

        let mother_ages = child_age.range_around_years(18, 40);

        // Can be simplified with `Iterator::collect_into` once it is stabilised.
        let pm_iter = potential_mothers
            .iter()
            .filter_map(|&(id, age)| mother_ages.contains(age).then_some(id));
        pm_in_age_range.clear();
        pm_in_age_range.extend(pm_iter);

        if let Some(mother_id) = pm_in_age_range.choose(rng).copied() {
            let mother = population.get(&mother_id).unwrap();
            let partner = mother.kinship.partner();

            set_as_parent_child(p_id, mother_id, &mut population);
            if let Some(partner_id) = partner {
                set_as_parent_child(p_id, partner_id, &mut population);
            }

            if child_age < Age::years(18) {
                set_as_guardian_dependent(mother_id, p_id, &mut population);
                set_as_provider_providee(mother_id, p_id, &mut population);
                if let Some(partner_id) = partner {
                    set_as_guardian_dependent(partner_id, p_id, &mut population);
                }
            }
        }
    }

    assert_eq!(population.len(), pars.population.init_size as usize);

    order
        .ids()
        .map(|id| population.remove(&id).unwrap())
        .collect()
}

pub fn init_class<R: Rng>(person: &mut Person, pars: &ModelPars, rng: &mut R) {
    let classes: Vec<_> = (0..N_CLASSES).collect();
    let class = *classes
        .choose_weighted(rng, |&i| pars.population.prob_classes[i])
        .unwrap() as u32;
    person.class.rank = Rank::new(class);
}

pub fn init_work<R: Rng>(person: &mut Person, pars: &ModelPars, rng: &mut R) {
    let age = person.basic.age;

    if age < Age::years(pars.work.age_teenagers) {
        person.work.status = WorkStatus::Child;
        return;
    }
    if age < Age::years(pars.work.age_adulthood) {
        person.work.status = WorkStatus::Teenager;
        return;
    }
    if age >= Age::years(pars.work.age_retirement) {
        person.work.status = WorkStatus::Retired;
        return;
    }

    let working_age = pars.work.start_working_age[person.class.rank_idx()];

    if age < Age::years(working_age) {
        person.work.status = WorkStatus::Student;
        return;
    }

    person.work.status = WorkStatus::FixedShiftEmployed;

    let mut working_time = 0.0;
    let age_years = age.year_month().0;
    for _ in working_age..=age_years {
        working_time *= pars.work.work_discounting_time;
        working_time += 12.0;
    }

    person.work.work_experience = working_time;
    person.work.working_periods = working_time;

    set_wage_progression(person, rng, pars);

    person.work.wage = compute_wage(person, rng, pars);
    let icare = person.care.index();
    let weekly_hours = pars.work.weekly_hours[icare] as f64;
    person.work.income = person.work.wage * weekly_hours;
    person.work.job_tenure = (1..=50).sample_single(rng).unwrap();
}

fn init_wealth(model: &mut Model, pars: &ModelPars) {
    for house in model.houses.values_mut().filter(|h| h.basic.is_occupied()) {
        house.income.cumulative_income = house
            .basic
            .occupants()
            .iter()
            .map(|&id| {
                let person = model.pop.alive(id);
                person.work.cumulative_income
            })
            .sum();
    }

    assign_wealth_by_inc_percentile_hh(model, pars);

    // assign household wealth to single members
    for house in model.houses.values_mut().filter(|h| h.basic.is_occupied()) {
        if house.income.cumulative_income > 0.0 {
            let earning_members: Vec<_> = house
                .basic
                .occupants()
                .iter()
                .filter_map(|&id| {
                    let member = model.pop.alive(id);
                    (member.work.cumulative_income > 0.0).then_some(id)
                })
                .collect();
            for id in earning_members {
                let member = model.pop.alive_mut(id);
                member.work.wealth = member.work.cumulative_income / house.income.cumulative_income
                    * house.income.wealth;
            }
        } else {
            let indep_members: Vec<_> = house
                .basic
                .occupants()
                .iter()
                .filter_map(|&id| {
                    let member = model.pop.alive_mut(id);
                    (!member.dependency.has_guardians()).then_some(id)
                })
                .collect();
            let nmembers = indep_members.len() as f64;
            for id in indep_members {
                let member = model.pop.alive_mut(id);
                member.work.wealth = house.income.wealth / nmembers;
            }
        }
    }
}

pub fn init_jobs(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let hired_agents: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| p.work.is_worker().then_some(p.id()))
        .collect();

    // FIXME: in Julia, the data passed is -1, which `assign_jobs` sets to a random month between 1
    // and 12. None of this is a valid `Date`, the date should be picked in the active period for
    // this agent.
    assign_jobs(&hired_agents, Date::new(0), pars, model);
    init_wealth(model, pars);
}

fn need_care(person: &Person, pars: &ModelPars) -> bool {
    person.basic.age < Age::years(pars.task_care.stop_child_care_age)
        || social_care_demand_per_day(person, pars) > 0
}

pub fn init_care(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let need_care: Vec<_> = model
        .pop
        .alives(order)
        // skip adolescents/adults that don't need care
        .filter_map(|p| need_care(p, pars).then_some(p.id()))
        .collect();

    need_care
        .into_iter()
        .for_each(|id| init_care_tasks(id, pars, model));
}
