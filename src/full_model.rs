use std::collections::{HashMap, HashSet};

use rand::{rngs::Xoshiro256PlusPlus, seq::SliceRandom};

use crate::{
    ModelPars,
    agents::{
        shifts::Shift,
        tasks::{IdTask, Task},
        towns::{IdTown, Town},
    },
    full_model::{
        data::{
            Post51Fertility, Post51MortalityFemale, Post51MortalityMale, Pre51Fertility,
            Pre51Mortality, Unemployment, WealthDistribution,
        },
        house::{House, IdHouse},
        person::{Id, Person},
    },
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
    utilities::Date,
};

pub mod data;
pub mod house;
pub mod person;

// FIXME: separate in smaller components to simplify borrowing and make it clearer which functions
// need what.
pub struct Model {
    /// Towns, containing houses.
    pub towns: HashMap<IdTown, Town>,
    /// Size of town grid, for plotting purposes.
    pub town_size: usize,
    /// Houses, located in towns.
    pub houses: HashMap<IdHouse, House>,
    /// The entire population. FIXME: where should dead people be kept?
    pub population: HashMap<Id, Person>,
    /// Shuffled list of population ids.
    pub shuffled_pop: Vec<Id>, // FIXME: make sure this is used where relevant
    /// Babies born in the current time step (moved to population at the end).
    pub babies: Vec<Person>,
    /// Probability distribution of shifts.
    pub shift_pool: Vec<Shift>,

    /// Set of all tasks.
    pub tasks: HashMap<IdTask, Task>,
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
    pub employed_pop_cache: HashSet<Id>,
    pub social_workers_cache: HashSet<Id>,
}

/// Create a model instance from parameters.
pub fn create_model(pars: &ModelPars) -> Model {
    // FIXME: allow fixed seed for testing, check for warm-up need
    let mut rng: Xoshiro256PlusPlus = rand::make_rng();

    let towns = create_towns(pars)
        .into_iter()
        .map(|town| (town.id(), town))
        .collect();

    // FIXME: how to make that first order reproducible? Might be ok when we use identity hash.
    let population = create_pyramid_population(pars, &mut rng);
    let shuffled_pop = population.iter().map(|p| p.id()).collect();

    let fert_post51 = Post51Fertility::read_from(&pars.data_files.fertility);

    let mut model = Model {
        towns,
        town_size: 0,
        houses: HashMap::new(), // FIXME: check capacity
        population: HashMap::with_capacity(population.len()),
        shuffled_pop,
        babies: Vec::new(),
        shift_pool: Vec::new(),
        tasks: HashMap::with_capacity(population.len() * 5), // FIXME: check pre alloc is useful
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
        employed_pop_cache: HashSet::new(),
        social_workers_cache: HashSet::new(),
    };

    initialise_houses_in_town(&mut model, population.len());
    assign_couples_to_houses(population, &mut model);

    for person in model.population.values_mut() {
        init_class(person, pars, &mut model.rng);
        init_work(person, pars, &mut model.rng);
    }

    init_jobs(&mut model, pars);
    init_care(&mut model, pars);

    model
}

/// Remove individual that died from the population.
pub fn remove_dead(model: &mut Model) {
    // Unclear what needs to be done here
    // Update shuffled pop only?
    // Also have dead in different bucket?
    // FIXME: either way, this should be enforced to be called right after death is handled
    todo!()
}

/// Perform one model step.
pub fn step_model(model: &mut Model, date: Date, pars: &ModelPars) {
    // avoid order effects
    // FIXME: could be done when dealing with dead people
    // FIXME: ensure this is used everywhere for iteration that's order-dependent
    model.shuffled_pop = model.population.keys().copied().collect();
    model.shuffled_pop.shuffle(&mut model.rng);
    // FIXME: avoid this clone
    let pop = model.shuffled_pop.clone();

    // pre-calc various population properties
    social_pre_calc(model);
    social_care_pre_calc(model, pars);
    divorce_pre_calc(model, pars);
    birth_pre_calc(model, pars);
    death_pre_calc(model, pars);
    job_pre_calc(date, model, pars);

    // run transitions
    // FIXME: move the `select_` into the transition themselves

    // death
    for &p_id in &pop {
        death(p_id, date, model, pars);
    }
    remove_dead(model);

    // adoption
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_assign_guardian(person, model) {
            assign_guardian(p_id, model);
        }
    }

    // birth
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_birth(person, model, pars) {
            birth(p_id, date, model, pars);
        }
    }

    // aging
    for &p_id in &pop {
        age_transition(p_id, model, pars);
    }

    update_income(model, pars);
    update_wealth(model, pars);

    house_ownership(model, pars);

    // hiring
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_unemployed(person) {
            unemployed_transition(p_id, date, model, pars);
        }
    }

    // firing
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_employed(person) {
            employed_transition(p_id, model, pars);
        }
    }

    // social care
    for &p_id in &pop {
        social_care_transition(p_id, model, pars);
    }

    compute_benefits(model, pars);

    // relocate
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_relocate(person, model) {
            relocate(p_id, model, pars);
        }
    }

    // sort new adults into students and workers
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_social_transition(person, pars) {
            social_transition(p_id, model, pars);
        }
    }

    // divorce
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_divorce(person) {
            divorce(p_id, date, model, pars);
        }
    }

    // marriage
    marriage_pre_calc(model, pars);
    for &p_id in &pop {
        let person = model.population.get(&p_id).unwrap();
        if select_marriage(person, pars) {
            marriage(p_id, model, pars);
        }
    }

    distribute_care(model, pars);

    for baby in model.babies.drain(..) {
        model.population.insert(baby.id(), baby);
    }
}
