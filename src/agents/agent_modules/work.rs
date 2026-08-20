use crate::{
    agents::shifts::Shift,
    full_model::{Model, person::Id},
    utilities::{Date, DayInWeek},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
    Child,
    Teenager,
    Student,
    /// Agent is employed at fixed hours.
    FixedShiftEmployed,
    /// Agent has flexible working hours, working when tasks are available.
    FlexibleShiftEmployed,
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
            WorkStatus::FlexibleShiftEmployed => true,
            WorkStatus::Retired => false,
            WorkStatus::Unemployed => false,
        }
    }

    // status index
    pub fn index(&self) -> usize {
        match self {
            WorkStatus::Child => 0,
            WorkStatus::Teenager => 1,
            WorkStatus::Student => 2,
            WorkStatus::FixedShiftEmployed => 3,
            WorkStatus::FlexibleShiftEmployed => 4,
            WorkStatus::Retired => 5,
            WorkStatus::Unemployed => 6,
        }
    }
}

pub struct Work {
    /// Work status.
    pub status: WorkStatus,
    pub initial_wage: f64,
    pub final_wage: f64,
    /// Hourly income for current job.
    pub wage: f64,
    /// Monthly income dependent on wage/work schedule.
    pub income: f64,
    pub cumulative_income: f64,
    pub disposable_income: f64,
    /// Last income, used for pension
    pub last_income: f64,
    pub wealth: f64,
    pub financial_wealth: f64,
    /// Potential working hours per week
    pub working_hours: u32,
    pub job_shift: Shift,
    pub days_off: Vec<DayInWeek>,
    /// Sum of actual working hours.
    pub available_working_hours: u32,
    /// Lifetime work.
    pub working_periods: f64,
    pub work_experience: f64,
    pub pension: f64,
    pub unemployment_months: u32,
    /// Period worked so far in current job.
    pub job_tenure: u32,
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
            initial_wage: 0.0,
            final_wage: 0.0,
            wage: 0.0,
            income: 0.0,
            cumulative_income: 0.0,
            disposable_income: 0.0,
            last_income: 0.0,
            wealth: 0.0,
            financial_wealth: 0.0,
            working_hours: 0,
            job_shift: Shift::empty(),
            days_off: Vec::new(),
            available_working_hours: 0,
            working_periods: 0.0,
            work_experience: 0.0,
            pension: 0.0,
            unemployment_months: 0,
            job_tenure: 0,
            month_hired: Date::new(0),
        }
    }
}

pub fn lose_job(p_id: Id, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);
    person.work.month_hired = Date::new(0);
    person.work.income = 0.0;
    person.work.working_hours = 0;
    person.work.job_shift = Shift::empty();
    person.work.job_tenure = 0;

    person
        .task
        .open_tasks
        .retain(|t_id| model.tasks.get(t_id).unwrap().is_care());
    person
        .task
        .assigned_tasks
        .retain(|t_id| model.tasks.get(t_id).unwrap().is_care());
    person
        .task
        .todo
        .iter_mut()
        .for_each(|todo_day| todo_day.retain(|t_id| model.tasks.get(t_id).unwrap().is_care()));

    // FIXME: should remove work tasks from model.tasks here?
}
