use identity_hash::IntMap;

use crate::{
    agents::tasks::{IdTask, Task},
    full_model::{
        Model,
        person::{Id, Person},
    },
    per_time::{Hourly, Weekly},
    utilities::{Date, HourInWeek},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
    Child,
    Teenager,
    Student,
    /// Agent is employed at fixed hours.
    FixedShiftEmployed,
    Retired,
    Unemployed,
}

impl WorkStatus {
    pub fn is_worker(&self) -> bool {
        match self {
            WorkStatus::Child => false,
            WorkStatus::Teenager => false,
            WorkStatus::Student => false,
            WorkStatus::FixedShiftEmployed => true,
            WorkStatus::Retired => false,
            WorkStatus::Unemployed => false,
        }
    }

    /// Base weekly care supply for this status.
    pub fn base_care_supply(&self) -> u32 {
        match self {
            WorkStatus::Child => 0,
            WorkStatus::Teenager => 10,
            WorkStatus::Student => 24,
            WorkStatus::FixedShiftEmployed => 32,
            WorkStatus::Retired => 60,
            WorkStatus::Unemployed => 48,
        }
    }
}

pub struct Work {
    /// Work status.
    pub status: WorkStatus,
    pub initial_wage: Hourly,
    pub final_wage: Hourly,
    /// Hourly income for current job.
    pub wage: Hourly,
    /// Income dependent on wage/work schedule.
    pub income: Weekly,
    pub cumulative_income: f64,
    pub disposable_income: Weekly,
    /// Last income, used for pension
    pub last_income: Weekly,
    pub wealth: f64,
    pub financial_wealth: f64,
    pub job_tasks: Vec<(IdTask, HourInWeek)>,
    /// Potential working hours per week
    pub working_hours: u32,
    /// Sum of actual working hours.
    pub available_working_hours: u32,
    /// Lifetime work.
    pub working_periods: f64,
    pub work_experience: f64,
    pub pension: Weekly,
    pub unemployment_months: u32,
    pub month_hired: Date,
}

impl Work {
    pub fn is_student(&self) -> bool {
        matches!(self.status, WorkStatus::Student)
    }

    pub fn is_worker(&self) -> bool {
        self.status.is_worker()
    }

    pub fn is_unemployed(&self) -> bool {
        matches!(self.status, WorkStatus::Unemployed)
    }
}

impl Default for Work {
    fn default() -> Self {
        Self {
            status: WorkStatus::Child,
            initial_wage: Hourly::zero(),
            final_wage: Hourly::zero(),
            wage: Hourly::zero(),
            income: Weekly::zero(),
            cumulative_income: 0.0,
            disposable_income: Weekly::zero(),
            last_income: Weekly::zero(),
            wealth: 0.0,
            financial_wealth: 0.0,
            job_tasks: Vec::with_capacity(40),
            working_hours: 0,
            available_working_hours: 0,
            working_periods: 0.0,
            work_experience: 0.0,
            pension: Weekly::zero(),
            unemployment_months: 0,
            month_hired: Date::new(0),
        }
    }
}

pub fn rm_job_tasks(person: &mut Person, task_pool: &mut IntMap<IdTask, Task>) {
    for (t_id, _) in person.work.job_tasks.drain(..) {
        let task = task_pool.remove(&t_id).unwrap();
        person.task.unschedule_task_if_present(&task);
    }
}

pub fn lose_job(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    person.work.month_hired = Date::new(0);
    person.work.income = Weekly::zero();
    person.work.working_hours = 0;
    rm_job_tasks(person, &mut model.tasks);
}
