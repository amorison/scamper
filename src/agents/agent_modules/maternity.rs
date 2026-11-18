use crate::utilities::Months;

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
}
