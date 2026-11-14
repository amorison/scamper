use crate::per_time::Weekly;

#[derive(Default)]
pub struct IncomeHouse {
    pub household_income: Weekly,
    pub income_per_capita: Weekly,
    pub cumulative_income: f64,
    pub wealth: f64,
    pub owned_by_occupants: bool,
    pub income_decile: u32, // FIXME: maybe wrapper type
    pub age_occupants: f64,
}
