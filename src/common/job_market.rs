use rand::{Rng, RngExt, seq::IndexedRandom};
use rand_distr::Uniform;

use crate::{
    ModelPars, N_AGE_BANDS, N_CLASSES,
    agents::{agent_modules::work::WorkStatus, shifts::Shift, tasks::Task},
    common::{income::compute_wage, social::change_status},
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    utilities::{Date, DayInWeek, HourInWeek, calc_rate_bias},
};

pub fn can_work(person: &Person) -> bool {
    person.care.need_level < 4 && !person.maternity.is_in_maternity()
}

pub fn is_working(person: &Person) -> bool {
    person.work.is_worker() && can_work(person)
}

pub fn is_unemployed(person: &Person) -> bool {
    matches!(person.work.status, WorkStatus::Unemployed) && can_work(person)
}

pub fn is_active(person: &Person) -> bool {
    is_working(person) || is_unemployed(person)
}

/// Build a person's weekly schedule based on their shift and working hours.
fn weekly_schedule<R: Rng>(shift: &Shift, weekly_hours: u32, rng: &mut R) -> [[bool; 24]; 7] {
    // TODO: are shifts always 5 days?
    let daily_hours = weekly_hours / 5;
    let reduced_hours = (shift.shift_hours.len() as u32).checked_sub(daily_hours);
    let shift_hours = match reduced_hours {
        None => &shift.shift_hours,
        Some(reduced_hours) => {
            let start_distr = Uniform::new_inclusive(0, reduced_hours).unwrap();
            let start = rng.sample(start_distr) as usize;
            let end = start + daily_hours as usize;
            &shift.shift_hours[start..end]
        }
    };

    let mut sched = [[false; 24]; 7];
    for day in &shift.days {
        for hour in shift_hours {
            sched[day.index()][hour.index()] = true
        }
    }

    sched
}

pub struct Shares {
    class: [f64; N_CLASSES],
    age_band: [[f64; N_AGE_BANDS]; N_CLASSES],
}

// TODO: fuse with class shares in social transition?
/// Count SES and age bands for active population.
pub fn calc_age_class_shares(model: &Model, order: &PopIterOrder) -> Shares {
    let mut class = [0.0; N_CLASSES];
    let mut age_band = [[0.0; N_AGE_BANDS]; N_CLASSES];

    // FIXME: double check that this is always called for the active subset of population
    // in the Julia code
    for person in model.pop.alives(order).filter(|p| is_active(p)) {
        let r = person.class.rank_idx();
        let a = person.basic.age.band();
        class[r] += 1.0;
        age_band[r][a] += 1.0;
    }

    // normalise age band shares by population per class
    for (r, cs) in class.iter().enumerate() {
        age_band[r].iter_mut().for_each(|a_s| *a_s /= cs);
    }

    // normalise class shares to full population
    let pop_size: f64 = class.iter().sum();
    class.iter_mut().for_each(|cs| *cs /= pop_size);

    Shares { class, age_band }
}

/// Unemployment rate per class and age.
pub fn compute_ur_by_class_age(
    ur: f64,
    shares: Shares,
    pars: &ModelPars,
) -> [[f64; N_AGE_BANDS]; N_CLASSES] {
    let mut rates = [[0.0; N_AGE_BANDS]; N_CLASSES];

    let class_bias: [_; N_CLASSES] =
        calc_rate_bias(|r| shares.class[r], pars.work.unemployment_class_bias);

    for r in 0..N_CLASSES {
        // calc normalisation factor for age bias
        let a_age: f64 = (0..N_AGE_BANDS)
            .map(|ab| shares.age_band[r][ab] * pars.work.unemployment_age_bias[ab])
            .sum();

        for age_group in 0..N_AGE_BANDS {
            let class_rate = ur * class_bias[r];
            let lower_age_band_rate = if a_age > 0.0 { class_rate / a_age } else { 0.0 };
            rates[r][age_group] = lower_age_band_rate * pars.work.unemployment_age_bias[age_group];
        }
    }

    rates
}

fn assign_job(p_id: Id, month: Date, shift: Shift, pars: &ModelPars, model: &mut Model) {
    change_status(p_id, WorkStatus::FixedShiftEmployed, model);
    let person = model.pop.alive_mut(p_id);
    person.work.unemployment_months = 0;
    person.work.month_hired = month;
    person.work.wage = compute_wage(person, &mut model.rng, pars);

    person.work.working_hours = pars.work.weekly_hours[person.care.need_level as usize];
    let job_schedule = weekly_schedule(&shift, person.work.working_hours, &mut model.rng);

    person.work.days_off = (0..7)
        .map(DayInWeek::new)
        .filter(|d| !shift.days.contains(d))
        .collect();
    person.work.job_shift = shift;

    for time in HourInWeek::all_hours() {
        let (day, hour) = time.day_hour();
        if job_schedule[day.index()][hour.index()] {
            let task = Task::work(person.id(), time);
            let t_id = task.id();
            model.tasks.insert(t_id, task);
            person.task.open_tasks.insert(t_id);
        }
    }
}

pub fn assign_jobs(hired_agents: &[Id], month: Date, pars: &ModelPars, model: &mut Model) {
    // TODO: draw without replacement?
    let n_shifts = model.shift_pool.len();
    let shifts: Vec<_> = (&mut model.rng)
        .sample_iter(Uniform::new(0, n_shifts).unwrap())
        .take(hired_agents.len())
        .map(|i| &model.shift_pool[i])
        .collect();

    // FIXME: this is equivalent to the Julia code, but seems strange.
    // Since we've already sampled `hired_agents.len()` shifts, this simply
    // shuffles them but doesn't make the socially attractive shifts more likely
    // to be taken. It seems like this sampling should be applied to the full
    // shift pool instead.
    let shifts: Vec<_> = shifts
        .sample_weighted(&mut model.rng, hired_agents.len(), |s| s.social_index)
        .unwrap()
        .copied()
        .cloned()
        .collect();

    assert_eq!(hired_agents.len(), shifts.len());

    for (i, shift) in shifts.into_iter().enumerate() {
        // FIXME: Julia version has a check if month = -1, then month = rand(1:12)
        // This doesn't seem to make much sense since month is a Date!

        let p_id = hired_agents[i];
        assign_job(p_id, month, shift, pars, model);
    }
}
