use std::io;

use crate::{
    ModelPars,
    analysis::Output,
    full_model::{create_model, step_model},
    utilities::Date,
};

pub fn main(pars: ModelPars) -> io::Result<()> {
    let mut out = Output::new(&pars)?;

    let (mut model, mut order) = create_model(&pars);

    let mut date = Date::new(pars.population.start_year);
    let finish = Date::new(pars.population.end_year);

    let npop = model.pop.size();
    println!("{date} {npop}");
    out.output(date, &model, &order)?;

    while date < finish {
        step_model(&mut model, &mut order, date, &pars);

        date = date.next_month();

        let npop = model.pop.size();
        let ndeads = model.pop.ndeads();
        let ntasks = model.tasks.len();
        println!("{date}, pop size: {npop}, ndeads: {ndeads}, ntasks: {ntasks}");
        out.output(date, &model, &order)?;
    }

    Ok(())
}
