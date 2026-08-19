use std::fmt::Display;

use identity_hash::{BuildIdentityHasher, IntMap};
use rand::{
    Rng, RngExt,
    distr::{Distribution, Uniform},
};

/// Date (accurate to the month)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(u32);

/// Age
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Age(u32);

/// Age interval
pub struct AgeRange(u32, u32);

/// Represent a month of the year
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MonthInYear(u32);

/// Represent a day of the week
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayInWeek(u32);

/// Hour in day from 00:00 (0-23 range)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HourInDay(u32);

/// Hour in week from 00:00 on Monday (0-167 range)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HourInWeek(u32);

impl Date {
    pub fn new(year: u32) -> Self {
        Self(year * 12)
    }

    /// Year and month (1-12) of this date.
    pub fn year_month(self) -> (u32, u32) {
        (self.0 / 12, self.0 % 12 + 1)
    }

    pub fn next_month(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (year, month) = self.year_month();
        write!(f, "{year}-{month:02}")
    }
}

impl Age {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn months(months: u32) -> Self {
        Self(months)
    }

    pub fn years(years: u32) -> Self {
        Self(years * 12)
    }

    /// Years and months (0-11) corresponding to this age.
    pub fn year_month(self) -> (u32, MonthInYear) {
        (self.0 / 12, MonthInYear(self.0 % 12))
    }

    /// Floating point number of years.
    pub fn years_f64(self) -> f64 {
        self.0 as f64 / 12.0
    }

    pub fn add_one_month(&mut self) {
        self.0 += 1;
    }

    /// Age category (0 to 5 included)
    pub fn band(self) -> usize {
        let (year, _) = self.year_month();
        if year <= 19 {
            0
        } else if year <= 54 {
            (year as usize - 5) / 10
        } else {
            5
        }
    }

    /// Age interval around this age
    pub fn range_around_years(&self, min_offset: i32, max_offset: i32) -> AgeRange {
        let min_offset = min_offset * 12;
        let max_offset = max_offset * 12;

        let age_min = self.0.saturating_add_signed(min_offset);
        let age_max = self.0.saturating_add_signed(max_offset);
        AgeRange(age_min, age_max)
    }
}

impl AgeRange {
    pub fn contains(&self, age: Age) -> bool {
        self.0 <= age.0 && age.0 < self.1
    }
}

impl MonthInYear {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl DayInWeek {
    /// Build a `DayIdx` from a day in [0, 6] range.
    pub fn new(day: u32) -> DayInWeek {
        assert!((0..7).contains(&day));
        DayInWeek(day)
    }

    /// Iterate through all the days in a week (7 values).
    pub fn all_days() -> impl Iterator<Item = DayInWeek> {
        (0..7).map(|h| Self(h))
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl HourInDay {
    pub fn new(hour: u32) -> HourInDay {
        HourInDay(hour % 24)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl HourInWeek {
    pub fn new(day: DayInWeek, hour: HourInDay) -> Self {
        Self(day.0 * 24 + hour.0)
    }

    /// Iterate through all the hours in a week (168 values).
    pub fn all_hours() -> impl Iterator<Item = HourInWeek> {
        (0..168).map(|h| HourInWeek(h))
    }

    pub fn day_hour(&self) -> (DayInWeek, HourInDay) {
        (DayInWeek(self.0 / 24), HourInDay(self.0 % 24))
    }
}

pub fn sum_class_bias<F, const N: usize>(f: F, bias: f64) -> f64
where
    F: Fn(usize) -> f64,
{
    (0..N).map(|c| f(c) * bias.powi(c as i32)).sum()
}

pub fn calc_rate_bias<F, const N: usize>(f: F, bias: f64) -> [f64; N]
where
    F: Fn(usize) -> f64,
{
    let sum = sum_class_bias::<_, N>(f, bias);
    let mut rates = [0.0; N];
    for c in 0..N {
        rates[c] = bias.powi(c as i32) / sum;
    }
    rates
}

/// Whether a random event of the given yearly probability should happen this month.
pub fn try_rand_yearly2monthly<R: Rng>(p_yearly: f64, rng: &mut R) -> bool {
    let dist = Uniform::new_inclusive(1, 12).unwrap();
    rng.random_bool(p_yearly) && dist.sample(rng) == 12
}

/// Create an `IntMap` with given capacity.
pub fn int_map_with_cap<K, V>(capacity: usize) -> IntMap<K, V> {
    IntMap::with_capacity_and_hasher(capacity, BuildIdentityHasher::new())
}
