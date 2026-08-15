use std::path::Path;

use serde::Deserialize;

use crate::{
    agents::agent_modules::basic_info::Gender,
    utilities::{Age, Date},
};

#[derive(Deserialize)]
struct AgePyramidRecord {
    male: f64,
    female: f64,
}

pub struct AgePyramid {
    male: Vec<f64>,
    female: Vec<f64>,
}

impl AgePyramid {
    pub fn of_gender(&self, gender: Gender) -> &[f64] {
        match gender {
            Gender::Female => &self.female,
            Gender::Male => &self.male,
        }
    }

    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::Reader::from_path(path).expect("error reading age pyramid");
        let records: Result<Vec<AgePyramidRecord>, _> = reader.deserialize().into_iter().collect();
        let records = records.expect("error reading age pyramid");
        let mut male = Vec::with_capacity(records.len());
        let mut female = Vec::with_capacity(records.len());
        for record in records.into_iter() {
            male.push(record.male);
            female.push(record.female);
        }
        Self { male, female }
    }
}

#[derive(Deserialize)]
struct Pre51FertilityRecord {
    year: u32,
    rate: f64,
}

pub struct Pre51Fertility(Vec<Pre51FertilityRecord>);

impl Pre51Fertility {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::Reader::from_path(path).expect("error reading pre51 fertility");
        let records: Result<Vec<Pre51FertilityRecord>, _> =
            reader.deserialize().into_iter().collect();
        Self(records.expect("error reading pre51 fertility"))
    }

    pub fn in_year(&self, year: u32) -> f64 {
        let iy = year.checked_sub(self.0[0].year).unwrap() as usize;
        self.0[iy].rate
    }
}

pub struct Post51Fertility(Vec<Vec<f64>>);

impl Post51Fertility {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .expect("error reading post51 fertility");
        let records: Result<Vec<Vec<f64>>, _> = reader.deserialize().into_iter().collect();
        Self(records.expect("error reading post51 fertility"))
    }

    pub fn rate_for(&self, ifert_age: usize, year: u32) -> f64 {
        // FIXME: double check the order of these indices
        self.0[ifert_age][(year - 1951) as usize]
    }

    /// Normalised age-specific fertility in 1951
    pub fn normalised_fertility1951(&self) -> Vec<f64> {
        // FIXME: double check the order of these indices
        let mut fert: Vec<_> = self.0.iter().map(|age| age[0]).collect();
        let norm = fert.iter().sum::<f64>() / fert.len() as f64;
        fert.iter_mut().for_each(|f| *f /= norm);
        fert
    }
}

#[derive(Deserialize)]
struct Pre51MortalityRecord {
    year: u32,
    rate: f64,
    infant_rate: f64,
}

pub struct Pre51Mortality(Vec<Pre51MortalityRecord>);

impl Pre51Mortality {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::Reader::from_path(path).expect("error reading pre51 mortality");
        let records: Result<Vec<Pre51MortalityRecord>, _> =
            reader.deserialize().into_iter().collect();
        Self(records.expect("error reading pre51 mortality"))
    }

    pub fn infant_rate(&self, date: Date) -> f64 {
        // FIXME: dodgy indexing
        let iyear = (date.year_month().0 - self.0[0].year) as usize;
        // infant rate is per 1k
        self.0[iyear].infant_rate / 1e3
    }

    pub fn rate(&self, date: Date) -> f64 {
        // FIXME: dodgy indexing
        let iyear = (date.year_month().0 - self.0[0].year) as usize;
        self.0[iyear].rate
    }
}

// FIXME: group these
pub struct Post51MortalityMale(Vec<Vec<f64>>);
pub struct Post51MortalityFemale(Vec<Vec<f64>>);

impl Post51MortalityMale {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .expect("error reading post51 male mortality");
        let records: Result<Vec<Vec<f64>>, _> = reader.deserialize().into_iter().collect();
        Self(records.expect("error reading post51 male mortality"))
    }

    pub fn rate(&self, age: Age, date: Date) -> f64 {
        // FIXME: dodgy indexing
        let iy = (date.year_month().0 - 1950) as usize;
        let iage = age.year_month().0.min(109) as usize;
        // FIXME: double check order of indices
        self.0[iage][iy]
    }
}

impl Post51MortalityFemale {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .expect("error reading post51 female mortality");
        let records: Result<Vec<Vec<f64>>, _> = reader.deserialize().into_iter().collect();
        Self(records.expect("error reading post51 female mortality"))
    }

    pub fn rate(&self, age: Age, date: Date) -> f64 {
        // FIXME: dodgy indexing
        let iy = (date.year_month().0 - 1950) as usize;
        let iage = age.year_month().0.min(109) as usize;
        // FIXME: double check order of indices
        self.0[iage][iy]
    }
}

pub struct Unemployment(Vec<f64>);

impl Unemployment {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .expect("error reading unemployment");
        let records: Result<Vec<f64>, _> = reader.deserialize().into_iter().collect();
        Self(records.expect("error reading unemployment"))
    }

    pub fn rate(&self, date: Date) -> f64 {
        // FIXME: dodgy indexing
        let iy = (date.year_month().0 - 1860) as usize;
        self.0[iy]
    }
}

pub struct WealthDistribution(Vec<f64>);

impl WealthDistribution {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Self {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .expect("error reading wealth distribution");
        let records: Result<Vec<f64>, _> = reader.deserialize().into_iter().collect();
        Self(records.expect("error reading wealth distribution"))
    }

    pub fn percentiles(&self) -> &[f64] {
        &self.0
    }
}
