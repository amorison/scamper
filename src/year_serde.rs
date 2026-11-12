use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::utilities::{Age, Date};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct Year(u32);

pub(crate) struct NotARoundYear;

impl Display for NotARoundYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("instant or duration should be a round year")
    }
}

impl From<Year> for Age {
    fn from(value: Year) -> Self {
        Age::years(value.0)
    }
}

impl TryFrom<&Age> for Year {
    type Error = NotARoundYear;

    fn try_from(value: &Age) -> Result<Self, Self::Error> {
        let (year, month) = value.year_month();
        if month.index() == 0 {
            Ok(Year(year))
        } else {
            Err(NotARoundYear)
        }
    }
}

impl From<Year> for Date {
    fn from(value: Year) -> Self {
        Date::new(value.0)
    }
}

impl TryFrom<&Date> for Year {
    type Error = NotARoundYear;

    fn try_from(value: &Date) -> Result<Self, Self::Error> {
        let (year, _) = value.year_month();
        if value.whole_year() {
            Ok(Year(year))
        } else {
            Err(NotARoundYear)
        }
    }
}

pub(crate) fn serialize<T, S, E>(value: &T, ser: S) -> Result<S::Ok, S::Error>
where
    for<'a> &'a T: TryInto<Year, Error = E>,
    E: Display,
    S: Serializer,
{
    match value.try_into() {
        Ok(year) => year.serialize(ser),
        Err(err) => Err(serde::ser::Error::custom(err)),
    }
}

pub(crate) fn deserialize<'de, T, D>(de: D) -> Result<T, D::Error>
where
    T: From<Year>,
    D: Deserializer<'de>,
{
    let year = Year::deserialize(de)?;
    Ok(year.into())
}
