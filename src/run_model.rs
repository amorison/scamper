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

    let finish = Date::new(pars.population.end_year);

    println!("{} {}", model.date, model.pop.size());
    out.output(&model, &order)?;

    while model.date < finish {
        step_model(&mut model, &mut order, &pars);

        let date = model.date;
        let npop = model.pop.size();
        let ndeads = model.pop.ndeads();
        let ntasks = model.tasks.len();
        println!("{date}, pop size: {npop}, ndeads: {ndeads}, ntasks: {ntasks}");
        out.output(&model, &order)?;
    }

    Ok(())
}
