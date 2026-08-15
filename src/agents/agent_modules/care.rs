#[derive(Default)]
pub struct Care {
    pub need_level: u32,
    pub social_work: u32,
    pub child_work: u32,
    pub wealth_spent_on_care: f64,
}

impl Care {
    pub fn index(&self) -> usize {
        self.need_level as usize
    }
}
