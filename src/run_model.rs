use std::{fs, io, path::PathBuf};

use jiff::{Zoned, tz::TimeZone};

use crate::{
    ModelPars,
    full_model::{create_model, step_model},
    utilities::Date,
};

pub fn main(pars: ModelPars) -> io::Result<()> {
    let now = Zoned::now()
        .with_time_zone(TimeZone::UTC)
        .strftime("%Y%m%d_%H%M%S");
    let run_dir = PathBuf::from(format!("lpm_run_{now}"));
    fs::create_dir(&run_dir)?;

    let out_pars = {
        let mut p = run_dir.clone();
        p.push("parameters.toml");
        p
    };
    pars.save_to(&out_pars)?;

    let (mut model, mut order) = create_model(&pars);

    let mut date = Date::new(pars.population.start_year);
    let finish = Date::new(pars.population.end_year);

    let npop = model.pop.size();
    println!("{date} {npop}");

    while date < finish {
        step_model(&mut model, &mut order, date, &pars);

        date = date.next_month();

        let npop = model.pop.size();
        let ndeads = model.pop.ndeads();
        let ntasks = model.tasks.len();
        println!("{date}, pop size: {npop}, ndeads: {ndeads}, ntasks: {ntasks}");
    }

    Ok(())
}
