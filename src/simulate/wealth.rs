use crate::{
    ModelPars, common::income::assign_wealth_by_inc_percentile, full_model::Model,
    population::PopIterOrder,
};

pub fn update_wealth(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    // julia comment:
    // Only workers: retired are assigned a wealth at the end of their working life (which they
    // consume thereafter).
    assign_wealth_by_inc_percentile(model, order, pars);

    // Calculate financial wealth from overall wealth
    // People without wage (pensioners) only consume financial wealth
    model.pop.for_each(order, |person| {
        if person.work.wage > 0.0 {
            person.work.financial_wealth = person.work.wealth * pars.work.share_financial_wealth;
        } else {
            debug_assert_eq!(person.work.wage, 0.0);
            person.work.financial_wealth -= person.care.wealth_spent_on_care;
            person.work.financial_wealth = person.work.financial_wealth.max(0.0);

            // passive income on wealth
            if person.work.cumulative_income > 0.0 {
                person.work.financial_wealth *= 1.0 + pars.work.pension_return_rate;
            }
        }
    });
}
