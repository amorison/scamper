use crate::{ModelPars, per_time::Weekly, utilities::Months};

#[derive(Default)]
pub struct Maternity {
    status: bool,
    months_since_birth: u32, // FIXME: proper wrapper type?
}

impl Maternity {
    pub fn is_in_maternity(&self) -> bool {
        self.status
    }

    /// Duration since birth.
    pub fn duration(&self) -> Months {
        Months::new(self.months_since_birth)
    }

    pub fn start(&mut self) {
        self.status = true;
        self.months_since_birth = 0;
    }

    pub fn step(&mut self) {
        self.months_since_birth += 1;
    }

    pub fn end(&mut self) {
        self.status = false;
        self.months_since_birth = 0;
    }

    pub fn adjust_income(&self, base_income: Weekly, pars: &ModelPars) -> Weekly {
        if !self.status {
            return base_income;
        }
        let maternity_income = pars.work.maternity_leave_income_factor * base_income;
        match self.months_since_birth {
            0..2 => maternity_income,
            2..10 => maternity_income.min(pars.work.min_statutory_maternity_pay),
            _ => Weekly::zero(),
        }
    }
}
