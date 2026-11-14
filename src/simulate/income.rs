use crate::{
    ModelPars,
    agents::agent_modules::work::WorkStatus,
    full_model::{Model, person::Person},
    per_time::Weekly,
    population::PopIterOrder,
};

fn update_person_income(person: &mut Person, pars: &ModelPars) {
    match person.work.status {
        WorkStatus::Child | WorkStatus::Teenager | WorkStatus::Student => {
            person.work.income = Weekly::zero()
        }
        WorkStatus::FixedShiftEmployed => {
            if person.maternity.is_in_maternity() {
                let expected_income = person
                    .work
                    .wage
                    .for_weekly_hours(pars.work.weekly_hours[person.care.index()]);
                let maternity_income = pars.work.maternity_leave_income_factor * expected_income;
                person.work.income = match person.maternity.duration() {
                    0..2 => maternity_income,
                    2..10 => maternity_income.min(pars.work.min_statutory_maternity_pay),
                    _ => Weekly::zero(),
                };
            } else {
                // FIXME: should be effectively worked hours
                person.work.income = person
                    .work
                    .wage
                    .for_weekly_hours(person.work.available_working_hours);
                person.work.last_income = person
                    .work
                    .wage
                    .for_weekly_hours(pars.work.weekly_hours[person.care.index()]);
            }
        }
        WorkStatus::Retired => {
            person.work.income = person.work.pension;
        }
        WorkStatus::Unemployed => person.work.income = Weekly::zero(),
    }

    person.work.disposable_income = person.work.income;
}

pub fn update_income(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    // Compute income from work based on last period job market and informal care
    // FIXME: include formal care if necessary?

    model.pop.for_each(order, |person| {
        update_person_income(person, pars);
    });

    for house in model.houses.values_mut().filter(|h| h.basic.is_occupied()) {
        house.income.household_income = house
            .basic
            .occupants()
            .iter()
            .map(|&p| model.pop.alive(p).work.income)
            .sum();
        house.income.income_per_capita =
            house.income.household_income / house.basic.occupants().len() as f64;
    }

    // Compute disposable income (i.e. after taxes and benefits)
    for p_id in order.ids() {
        let person = model.pop.alive_mut(p_id);
        if person.work.income <= Weekly::zero() {
            continue;
        }
        let mut employee_pension_contribution = Weekly::zero();
        let pension_min = Weekly::new(162.0);
        let pension_max = Weekly::new(893.0);
        if person.work.disposable_income > pension_min {
            if person.work.disposable_income < pension_max {
                employee_pension_contribution =
                    (person.work.disposable_income - pension_min) * 0.12;
            } else {
                employee_pension_contribution = (pension_max - pension_min) * 0.12;
                employee_pension_contribution +=
                    (person.work.disposable_income - pension_max) * 0.02;
            }
        }
        person.work.disposable_income -= employee_pension_contribution;

        let mut tax = Weekly::zero();
        let mut residual_income = person.work.disposable_income;
        for (i, &taxb) in pars.work.tax_brackets.iter().enumerate() {
            if residual_income > taxb {
                let taxable = residual_income - taxb;
                tax += taxable * pars.work.taxation_rates[i];
                residual_income -= taxable;
            }
        }
        person.work.disposable_income -= tax;
    }

    model.pop.for_each(order, |person| {
        person.work.disposable_income += person.benefits.benefits;
        person.work.cumulative_income += person.work.disposable_income.per_month();
    });
}
