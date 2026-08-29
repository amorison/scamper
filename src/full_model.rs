use identity_hash::IntMap;
use rand::{RngExt, SeedableRng, rngs::Xoshiro256PlusPlus};

use crate::{
    ModelPars,
    agents::{
        shifts::ShiftPool,
        tasks::{Carer, IdTask, Task},
        towns::{IdTown, Town},
    },
    full_model::{
        data::{
            Post51Fertility, Post51MortalityFemale, Post51MortalityMale, Pre51Fertility,
            Pre51Mortality, Unemployment, WealthDistribution,
        },
        house::{House, IdHouse},
    },
    population::{PopIterOrder, Population},
    setup::{
        map::{create_towns, initialise_houses_in_town},
        map_pop::assign_couples_to_houses,
        population::{create_pyramid_population, init_care, init_class, init_jobs, init_work},
    },
    simulate::{
        age::age_transition,
        benefits::compute_benefits,
        birth::{BirthCache, birth, birth_pre_calc, select_birth},
        death::{DeathCache, death, death_pre_calc},
        dependencies::{assign_guardian, select_assign_guardian},
        divorce::{DivorceCache, divorce, divorce_pre_calc, select_divorce},
        housing_top_down::house_ownership,
        income::update_income,
        job_transition::{
            JobCache, employed_transition, job_pre_calc, select_employed, select_unemployed,
            unemployed_transition,
        },
        marriage::{MarriageCache, marriage, marriage_pre_calc, select_marriage},
        relocate::{relocate, select_relocate},
        social::{SocialCache, select_social_transition, social_pre_calc, social_transition},
        social_care::{SocialCareCache, social_care_pre_calc, social_care_transition},
        tasks_care::distribute_care,
        wealth::update_wealth,
    },
    utilities::{Date, int_map_with_cap},
};

pub mod data;
pub mod house;
pub mod person;

// FIXME: separate in smaller components to simplify borrowing and make it clearer which functions
// need what.
pub struct Model {
    /// Towns, containing houses.
    pub towns: IntMap<IdTown, Town>,
    /// Size of town grid, for plotting purposes.
    pub town_size: usize,
    /// Houses, located in towns.
    pub houses: IntMap<IdHouse, House>,
    /// The entire population.
    pub pop: Population,
    /// Probability distribution of shifts.
    pub shift_pool: ShiftPool,

    /// Set of all tasks.
    pub tasks: IntMap<IdTask, Task>,
    /// Random number generator.
    pub rng: Xoshiro256PlusPlus,

    // Some empirical data used in the model.
    pub fert_f_by_age_51: Vec<f64>,
    pub fert_pre51: Pre51Fertility,
    pub fert_post51: Post51Fertility,
    pub pre51_deaths: Pre51Mortality,
    pub female_deaths: Post51MortalityFemale,
    pub male_deaths: Post51MortalityMale,
    pub unemployment_series: Unemployment,
    pub wealth_percentiles: WealthDistribution,

    // Caches used by some of the transitions.
    pub birth_cache: BirthCache,
    pub death_cache: DeathCache,
    pub marriage_cache: MarriageCache,
    pub social_cache: SocialCache,
    pub social_care_cache: SocialCareCache,
    pub divorce_cache: DivorceCache,
    pub job_cache: JobCache,
}

fn init_rng(reproducible_rng: bool) -> Xoshiro256PlusPlus {
    let mut rng: Xoshiro256PlusPlus = if reproducible_rng {
        Xoshiro256PlusPlus::seed_from_u64(1)
    } else {
        rand::make_rng()
    };

    // Consume the first 100 64-bit values to ensure we're
    // out of zeroland: https://doi.org/10.1145/3460772
    (&mut rng).random_iter::<u64>().nth(100);

    rng
}

/// Create a model instance from parameters.
pub fn create_model(pars: &ModelPars) -> (Model, PopIterOrder) {
    let mut rng: Xoshiro256PlusPlus = init_rng(pars.init.reproducible_rng);

    let towns = create_towns(pars)
        .into_iter()
        .map(|town| (town.id(), town))
        .collect();

    let (pop, mut order) = Population::for_npersons(pars.population.init_size as usize);
    let population = create_pyramid_population(pars, &mut order, &mut rng);

    let fert_post51 = Post51Fertility::read_from(&pars.data_files.fertility);

    let mut model = Model {
        towns,
        town_size: 0,
        houses: int_map_with_cap(0), // FIXME: check capacity
        pop,
        shift_pool: ShiftPool::new(pars, &mut rng),
        tasks: int_map_with_cap(population.len() * 5),
        rng,
        fert_f_by_age_51: fert_post51.normalised_fertility1951(),
        fert_pre51: Pre51Fertility::read_from(&pars.data_files.pre51_fertility),
        fert_post51,
        pre51_deaths: Pre51Mortality::read_from(&pars.data_files.pre51_deaths),
        female_deaths: Post51MortalityFemale::read_from(&pars.data_files.death_female),
        male_deaths: Post51MortalityMale::read_from(&pars.data_files.death_male),
        unemployment_series: Unemployment::read_from(&pars.data_files.unemployment_rate),
        wealth_percentiles: WealthDistribution::read_from(&pars.data_files.wealth_distribution),
        birth_cache: BirthCache::default(),
        death_cache: DeathCache::default(),
        marriage_cache: MarriageCache::default(),
        social_cache: SocialCache::default(),
        social_care_cache: SocialCareCache::default(),
        divorce_cache: DivorceCache::default(),
        job_cache: JobCache::default(),
    };

    initialise_houses_in_town(&mut model, population.len());
    assign_couples_to_houses(population, &mut model);

    model.pop.for_each(&order, |person| {
        init_class(person, pars, &mut model.rng);
        init_work(person, pars, &mut model.rng);
    });

    init_jobs(&mut model, &order, pars);
    init_care(&mut model, &order, pars);

    (model, order)
}

/// Perform one model step.
pub fn step_model(model: &mut Model, order: &mut PopIterOrder, date: Date, pars: &ModelPars) {
    // run transitions
    // FIXME: move the `select_` into the transition themselves

    // FIXME: make cache up-to-date by construction, otherwise
    // this is very brittle... death needs social class shares,
    // hence why `social_pre_calc` is called both before and
    // after death.
    social_pre_calc(model, order);
    death_pre_calc(model, order, pars);
    for p_id in order.ids() {
        death(p_id, date, model, pars);
    }
    order.register_dead(&mut model.pop);

    // pre-calc various population properties
    social_pre_calc(model, order);
    social_care_pre_calc(model, pars);
    divorce_pre_calc(model, pars);
    birth_pre_calc(model, order, pars);
    job_pre_calc(date, model, order, pars);

    // adoption
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_assign_guardian(person) {
            assign_guardian(p_id, model, order);
        }
    }

    // birth
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_birth(person, model, pars) {
            birth(p_id, date, model, pars);
        }
    }

    // aging
    for p_id in order.ids() {
        age_transition(p_id, model, pars);
    }

    update_income(model, order, pars);
    update_wealth(model, order, pars);

    house_ownership(model, pars);

    // hiring
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_unemployed(person) {
            unemployed_transition(p_id, date, model, pars);
        }
    }

    // firing
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_employed(person) {
            employed_transition(p_id, model, pars);
        }
    }

    // social care
    for p_id in order.ids() {
        social_care_transition(p_id, model, pars);
    }

    compute_benefits(model, order, pars);

    // relocate
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_relocate(person, model) {
            relocate(p_id, model, pars);
        }
    }

    // sort new adults into students and workers
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_social_transition(person, pars) {
            social_transition(p_id, model, pars);
        }
    }

    // divorce
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_divorce(person) {
            divorce(p_id, date, model, pars);
        }
    }

    // marriage
    marriage_pre_calc(model, order, pars);
    for p_id in order.ids() {
        let person = model.pop.alive(p_id);
        if select_marriage(person, pars) {
            marriage(p_id, model, pars);
        }
    }

    // Accept work tasks by default, which might then be replaced by care tasks.
    for p_id in order.ids() {
        let person = model.pop.alive_mut(p_id);
        for &t_id in &person.work.job_tasks {
            let task = model.tasks.get_mut(&t_id).unwrap();
            // FIXME: should be unnecessary for work tasks
            task.worker = Some(Carer::Person(p_id));
            if person.how_busy_at(task.time) <= 0.0 {
                person.task.schedule_task(task);
            }
        }
    }

    distribute_care(model, order, pars);

    // include babies in the population iteration
    order.register_new(&mut model.pop, &mut model.rng);
}
