#[derive(Default)]
pub struct IncomeHouse {
    pub household_income: f64,
    pub disposable_income: f64,
    pub income_per_capita: f64,
    pub cumulative_income: f64,
    pub wealth: f64,
    pub owned_by_occupants: bool,
    pub income_decile: u32, // FIXME: maybe wrapper type
    pub age_occupants: f64,
}
