use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign},
};

use serde::{Deserialize, Serialize};

const WEEKS_PER_MONTH: f64 = 365.24 / 7.0 / 12.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Weekly(f64);

impl Weekly {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub const fn from_monthly(value: f64) -> Self {
        Self(value / WEEKS_PER_MONTH)
    }

    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub const fn per_month(self) -> f64 {
        self.0 * WEEKS_PER_MONTH
    }

    pub const fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub const fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl Add<Weekly> for Weekly {
    type Output = Weekly;

    fn add(self, rhs: Weekly) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Weekly> for Weekly {
    type Output = Weekly;

    fn sub(self, rhs: Weekly) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign<Weekly> for Weekly {
    fn add_assign(&mut self, rhs: Weekly) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Weekly> for Weekly {
    fn sub_assign(&mut self, rhs: Weekly) {
        self.0 -= rhs.0;
    }
}

impl Sum<Weekly> for Weekly {
    fn sum<I: Iterator<Item = Weekly>>(iter: I) -> Self {
        iter.fold(Weekly::zero(), Add::add)
    }
}

impl Mul<f64> for Weekly {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<Weekly> for f64 {
    type Output = Weekly;

    fn mul(self, rhs: Weekly) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Weekly {
    type Output = Weekly;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Div<Weekly> for Weekly {
    type Output = f64;

    fn div(self, rhs: Weekly) -> Self::Output {
        self.0 / rhs.0
    }
}

impl DivAssign<f64> for Weekly {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Hourly(f64);

impl Hourly {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub const fn for_weekly_hours(self, hours: u32) -> Weekly {
        Weekly::new(self.0 * hours as f64)
    }
}

impl Mul<f64> for Hourly {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<Hourly> for f64 {
    type Output = Hourly;

    fn mul(self, rhs: Hourly) -> Self::Output {
        rhs * self
    }
}

impl Div<Hourly> for Hourly {
    type Output = f64;

    fn div(self, rhs: Hourly) -> Self::Output {
        self.0 / rhs.0
    }
}
