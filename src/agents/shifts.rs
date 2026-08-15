use crate::utilities::{DayInWeek, HourInDay, HourInWeek};

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

    pub fn in_shift(&self, hiw: HourInWeek) -> bool {
        let (day, hour) = hiw.day_hour();
        self.days.contains(&day) && self.shift_hours.contains(&hour)
    }
}
