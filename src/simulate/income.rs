use crate::{
    ModelPars,
    agents::agent_modules::work::WorkStatus,
    full_model::{Model, person::Person},
    population::PopIterOrder,
};

fn update_person_income(person: &mut Person, pars: &ModelPars) {
    match person.work.status {
        WorkStatus::Child | WorkStatus::Teenager | WorkStatus::Student => person.work.income = 0.0,
        WorkStatus::FixedShiftEmployed | WorkStatus::FlexibleShiftEmployed => {
            if person.maternity.is_in_maternity() {
                let expected_income =
                    person.work.wage * pars.work.weekly_hours[person.care.index()] as f64;
                let maternity_income = pars.work.maternity_leave_income_factor * expected_income;
                person.work.income = match person.maternity.duration() {
                    0..2 => maternity_income,
                    2..10 => maternity_income.min(pars.work.min_statutory_maternity_pay),
                    _ => 0.0,
                };
            } else {
                // FIXME: should be effectively worked hours
                person.work.income = person.work.wage * person.work.available_working_hours as f64;
                // FIXME: what is this?
                person.work.last_income =
                    person.work.wage * pars.work.weekly_hours[person.care.index()] as f64;
            }
        }
        WorkStatus::Retired => {
            person.work.income = person.work.pension;
        }
        WorkStatus::Unemployed => person.work.income = 0.0,
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
        if person.work.income <= 0.0 {
            continue;
        }
        let mut employee_pension_contribution = 0.0;
        if person.work.disposable_income > 162.0 {
            if person.work.disposable_income < 893.0 {
                employee_pension_contribution = (person.work.disposable_income - 162.0) * 0.12;
            } else {
                employee_pension_contribution = (893.0 - 162.0) * 0.12;
                employee_pension_contribution += (person.work.disposable_income - 893.0) * 0.02;
            }
        }
        person.work.disposable_income -= employee_pension_contribution;

        let mut tax = 0.0;
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
        // FIXME: make sure this is updated in the correct order
        person.work.disposable_income += person.benefits.benefits;
        person.work.cumulative_income += person.work.disposable_income;
    });
}
