use crate::per_time::Weekly;

#[derive(Default)]
pub struct Benefits {
    pub benefits: Weekly,
    pub highest_disability: bool,
    pub uc: bool,
    pub guarantee_credit: bool,
}
