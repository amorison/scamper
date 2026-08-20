use crate::utilities::{DayInWeek, HourInDay};

#[derive(Debug, Clone)]
pub struct Shift {
    pub days: Vec<DayInWeek>,
    pub start: HourInDay,
    pub start_index: u32,
    pub shift_hours: Vec<HourInDay>,
    pub finish: HourInDay,
    pub social_index: f64,
}

impl Shift {
    pub fn empty() -> Self {
        Shift {
            days: Vec::new(),
            start: HourInDay::new(0),
            start_index: 0,
            shift_hours: Vec::new(),
            finish: HourInDay::new(0),
            social_index: 0.0,
        }
    }
}
