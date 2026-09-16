use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use jiff::{Zoned, tz::TimeZone};
use serde::Serialize;

use crate::{
    ModelPars, N_AGE_YEARS, N_CARE_LEVELS, N_CLASSES, agents::agent_modules::basic_info::Gender,
    full_model::Model, population::PopIterOrder, utilities::Date,
};

pub struct Output {
    main_csv: csv::Writer<File>,
    class_tally_csv: csv::Writer<File>,
    care_tally_csv: csv::Writer<File>,
    pyramid_male_csv: csv::Writer<File>,
    pyramid_female_csv: csv::Writer<File>,
}

impl Output {
    pub fn new(pars: &ModelPars) -> io::Result<Self> {
        let now = Zoned::now()
            .with_time_zone(TimeZone::UTC)
            .strftime("%Y%m%d_%H%M%S");
        let run_dir = PathBuf::from(format!("lpm_run_{now}"));
        fs::create_dir(&run_dir)?;

        let out_pars = run_dir.join("parameters.toml");
        pars.save_to(&out_pars)?;

        let main_csv = csv::Writer::from_path(run_dir.join("stats.csv"))?;
        let class_tally_csv = csv::Writer::from_path(run_dir.join("class.csv"))?;
        let care_tally_csv = csv::Writer::from_path(run_dir.join("care_need.csv"))?;
        let pyramid_male_csv = csv::Writer::from_path(run_dir.join("pyramid_male.csv"))?;
        let pyramid_female_csv = csv::Writer::from_path(run_dir.join("pyramid_female.csv"))?;

        Ok(Self {
            main_csv,
            class_tally_csv,
            care_tally_csv,
            pyramid_male_csv,
            pyramid_female_csv,
        })
    }

    pub fn output(&mut self, date: Date, model: &Model, order: &PopIterOrder) -> io::Result<()> {
        let stats = PopulationStats::new(model, order);
        let main_rec = MainRecord::new(date, &stats);
        self.main_csv.serialize(main_rec)?;

        let (year, month) = date.year_month();
        let year = format!("{year}");
        let month = format!("{month}");

        self.class_tally_csv.write_field(&year)?;
        self.class_tally_csv.write_field(&month)?;
        self.class_tally_csv.serialize(stats.class_tally)?;

        self.care_tally_csv.write_field(&year)?;
        self.care_tally_csv.write_field(&month)?;
        self.care_tally_csv.serialize(stats.care_level_tally)?;

        self.pyramid_male_csv.write_field(&year)?;
        self.pyramid_male_csv.write_field(&month)?;
        self.pyramid_male_csv
            .serialize(stats.age_pyramid_male.to_vec())?;

        self.pyramid_female_csv.write_field(&year)?;
        self.pyramid_female_csv.write_field(&month)?;
        self.pyramid_female_csv
            .serialize(stats.age_pyramid_female.to_vec())?;

        if date.whole_year() {
            self.main_csv.flush()?;
            self.class_tally_csv.flush()?;
            self.care_tally_csv.flush()?;
            self.pyramid_male_csv.flush()?;
            self.pyramid_female_csv.flush()?;
        }
        Ok(())
    }
}

struct PopulationStats {
    pop_size: usize,
    n_married: usize,
    n_working: usize,
    n_unemployed: usize,
    class_tally: [usize; N_CLASSES],
    age_pyramid_male: [usize; N_AGE_YEARS],
    age_pyramid_female: [usize; N_AGE_YEARS],
    care_level_tally: [usize; N_CARE_LEVELS],
    n_care_home_occupied: usize,
    n_care_home_capacity: usize,
}

impl PopulationStats {
    fn new(model: &Model, order: &PopIterOrder) -> Self {
        let mut stats = Self {
            pop_size: model.pop.size(),
            n_married: 0,
            n_working: 0,
            n_unemployed: 0,
            class_tally: [0; N_CLASSES],
            age_pyramid_male: [0; _],
            age_pyramid_female: [0; _],
            care_level_tally: [0; N_CARE_LEVELS],
            n_care_home_occupied: model.carehomes.occupied(),
            n_care_home_capacity: model.carehomes.capacity(),
        };
        for person in model.pop.alives(order) {
            if !person.kinship.is_single() {
                stats.n_married += 1;
            }

            if person.work.is_worker() {
                stats.n_working += 1;
            } else if person.work.is_unemployed() {
                stats.n_unemployed += 1;
            }

            stats.class_tally[person.class.rank_idx()] += 1;

            let iage = person.basic.age.year_month().0 as usize;
            match person.basic.gender {
                Gender::Female => stats.age_pyramid_female[iage] += 1,
                Gender::Male => stats.age_pyramid_male[iage] += 1,
            }

            stats.care_level_tally[person.care.index()] += 1;
        }
        stats
    }
}

#[derive(Serialize)]
struct MainRecord {
    year: u32,
    month: u32,
    pop_size: usize,
    n_married: usize,
    n_working: usize,
    n_unemployed: usize,
    n_care_home_occupied: usize,
    n_care_home_capacity: usize,
}

impl MainRecord {
    fn new(date: Date, stats: &PopulationStats) -> Self {
        let (year, month) = date.year_month();
        Self {
            year,
            month,
            pop_size: stats.pop_size,
            n_married: stats.n_married,
            n_working: stats.n_working,
            n_unemployed: stats.n_unemployed,
            n_care_home_occupied: stats.n_care_home_occupied,
            n_care_home_capacity: stats.n_care_home_capacity,
        }
    }
}
