use crate::{
    ModelPars,
    agents::{
        agent_modules::basic_info::Gender,
        interactions::{family::has_own_children_at_home, housing::living_together},
    },
    full_model::{
        Model,
        house::House,
        person::{Id, Person},
    },
    population::PopIterOrder,
    utilities::Age,
};

pub fn compute_benefits(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    model.pop.for_each(order, |agent| {
        agent.benefits.benefits = 0.0;
        agent.benefits.highest_disability = false;
        agent.benefits.uc = false;
    });

    child_benefits(model, order, pars);
    disability_benefits(model, order, pars);
    universal_credit(model, order, pars);
    pension_credit(model, order, pars);
}

fn child_benefits(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let income_threshold = pars.benefit.child_benefit_income_threshold;
    for p_id in order.ids() {
        let parent = model.pop.alive_mut(p_id);
        if !parent.dependency.has_dependents() {
            continue;
        }

        let n_deps = parent.dependency.dependents.len();
        let potential_benefits = pars.benefit.first_child_benefit
            + pars.benefit.other_children_benefit * (n_deps - 1) as f64;

        if let Some(partner_id) = parent.kinship.partner() {
            let eligible = parent.work.income < income_threshold;
            let partner = model.pop.alive(partner_id);
            let partner_income = partner.work.income;
            if eligible || partner_income < income_threshold {
                let parent = model.pop.alive_mut(p_id);
                parent.benefits.benefits += potential_benefits / 2.0;
            }
        } else {
            if parent.work.income < income_threshold {
                parent.benefits.benefits += potential_benefits;
            }
        }
    }
}

fn disability_benefits(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    // FIXME: clarify indexing of care levels from pars.
    // Arrays in pars should probably have a N_CARE_LEVELS size with repetition where needed to
    // reduce confusion since indexing is not trivial here.
    for p_id in order.ids() {
        let person = model.pop.alive_mut(p_id);

        // children
        if person.basic.age < Age::years(16) && person.care.need_level > 0 {
            let idla = person.care.index() / 2;
            let im = (person.care.index() + 1) / 2 - 1;
            person.benefits.benefits += pars.benefit.care_dla[idla] + pars.benefit.mobility_dla[im];
            person.benefits.highest_disability = person.care.need_level > 3;
            continue;
        }

        // PIP
        if person.basic.age < Age::years(pars.work.age_retirement) && person.care.need_level > 0 {
            let ipip = (person.care.index() + 1) / 2 - 1;
            person.benefits.benefits += pars.benefit.care_pip[ipip];
            if person.care.need_level > 1 {
                let im = person.care.index() / 2 - 1;
                person.benefits.benefits += pars.benefit.mobility_pip[im];
            }
            person.benefits.highest_disability = person.care.need_level > 2;
        }

        // attendance allowance
        if person.basic.age >= Age::years(pars.work.age_retirement) && person.care.need_level > 2 {
            person.benefits.benefits += if person.care.need_level == 3 {
                pars.benefit.care_aa[0]
            } else {
                pars.benefit.care_aa[1]
            };
            // FIXME: do not set person.benefits.highest_disability?
        }

        // carer's allowance
        if person.care.social_work >= 35 {
            person.benefits.benefits += pars.benefit.carers_allowance;
        }
    }
}

fn is_uc_eligible_adult(person: &Person, pars: &ModelPars) -> bool {
    person.basic.age >= Age::years(18)
        && person.basic.age < Age::years(pars.work.age_retirement)
        && (person.work.is_worker() || person.work.is_unemployed())
}

fn is_uc_eligible_student(person: &Person, model: &Model, pars: &ModelPars) -> bool {
    person.work.is_student()
        && (person.dependency.has_dependents()
            || person.basic.age >= Age::years(pars.work.age_retirement)
            || person.care.need_level > 0
            || person
                .partner(model)
                .is_some_and(|partner| partner.benefits.uc))
}

fn is_uc_eligible_young(person: &Person, model: &Model) -> bool {
    person.basic.age >= Age::years(16)
        && person.basic.age < Age::years(18)
        && ((person.work.is_student()
            && person.kinship.father.is_none()
            && person.kinship.mother.is_none())
            || person.care.social_work >= 35
            || has_own_children_at_home(person, model))
}

fn universal_credit(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    // condition 1: age between 18 and 64
    // condition 2: low income or unemployed
    // condition 3: savings less than 16_000

    let eligible_adults: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| is_uc_eligible_adult(p, pars).then_some(p.id()))
        .collect();

    eligible_adults
        .into_iter()
        .for_each(|p_id| compute_uc(p_id, model, pars));

    // need to do that afterwards, so that partners have been processed
    let eligible_y_std: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| {
            (is_uc_eligible_student(p, model, pars) || is_uc_eligible_young(p, model))
                .then_some(p.id())
        })
        .collect();

    eligible_y_std
        .into_iter()
        .for_each(|p_id| compute_uc(p_id, model, pars));

    // housing element (for renters): LHA rate based on household composition (number of rooms).
    // 1 room for each of the following:
    // - The household's head and partner
    // - Any other person over 16, as long as they aren't living with you as your tenant
    // - Two children under 16 of the same gender
    // - Two children under 10
    // - Any other child under 16
    let eligible_housing: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| {
            let house = p.house(model);
            let eligible =
                p.benefits.uc && !house.income.owned_by_occupants && !p.dependency.is_dependent();
            if !eligible {
                return None;
            }
            let town = house.town(model);
            let iroom = compute_max_rooms(house, model) - 1;
            let benefit = if p
                .partner(&model)
                .map(|partner| partner.benefits.uc)
                .unwrap_or(true)
            {
                town.lha[iroom]
            } else {
                town.lha[iroom] / 2.0
            };
            Some((p.id(), benefit))
        })
        .collect();
    for (p_id, benefit) in eligible_housing {
        let person = model.pop.alive_mut(p_id);
        person.benefits.benefits += benefit;
    }
}

fn is_dep(p: &Person) -> bool {
    p.basic.age < Age::years(16) || (p.basic.age < Age::years(20) && p.work.is_student())
}

fn n_dependents(house: &House, model: &Model) -> usize {
    house
        .basic
        .occupants()
        .iter()
        .map(|&p_id| model.pop.alive(p_id))
        .filter(|&p| is_dep(p))
        .count()
}

fn n_mildly_disabled_dependents(house: &House, model: &Model) -> usize {
    house
        .basic
        .occupants()
        .iter()
        .map(|&p_id| model.pop.alive(p_id))
        .filter(|&p| is_dep(p) && p.care.need_level > 0 && !p.benefits.highest_disability)
        .count()
}

fn n_crit_disabled_dependents(house: &House, model: &Model) -> usize {
    house
        .basic
        .occupants()
        .iter()
        .map(|&p_id| model.pop.alive(p_id))
        .filter(|&p| is_dep(p) && p.benefits.highest_disability)
        .count()
}

fn compute_uc(p_id: Id, model: &mut Model, pars: &ModelPars) {
    let person = model.pop.alive(p_id);
    let (partner_fw, partner_income, partner_below25) = if let Some(partner) = person.partner(model)
    {
        (
            partner.work.financial_wealth,
            partner.work.income,
            partner.basic.age < Age::years(25),
        )
    } else {
        (0.0, 0.0, false)
    };

    let total_wealth = person.work.financial_wealth + partner_fw;
    if total_wealth >= pars.benefit.capital_high_threshold {
        return;
    }

    let total_income = person.work.income + partner_income;

    let mut uc_income = total_income
        + pars.benefit.capital_income
            * ((person.work.financial_wealth - pars.benefit.capital_low_threshold)
                / pars.benefit.saving_uc_rate)
                .floor()
                .max(0.0);

    let house = model.houses.get(&person.house).unwrap();
    let n_uc_deps = n_dependents(house, model);
    let n_mildly_disabled_uc_deps = n_mildly_disabled_dependents(house, model);
    let n_crit_disabled_uc_deps = n_crit_disabled_dependents(house, model);

    // TODO: dependence needs to be done properly
    if (!person.dependency.is_dependent() && n_uc_deps > 0) || person.care.need_level > 0 {
        if !house.income.owned_by_occupants {
            // assuming the agent with get the housing cost element
            uc_income = (uc_income - pars.benefit.work_allowance_hs).max(0.0);
        } else {
            uc_income = (uc_income - pars.benefit.work_allowance_no_hs).max(0.0);
        }
    }

    let uc_reduction = uc_income * pars.benefit.income_reduction;
    let benefit = if person.kinship.is_single() {
        if person.basic.age < Age::years(25) {
            (pars.benefit.single_below25 - uc_reduction).max(0.0)
        } else {
            (pars.benefit.single_25plus - uc_reduction).max(0.0)
        }
    } else {
        if person.basic.age < Age::years(25) && partner_below25 {
            (pars.benefit.couple_below25 - uc_reduction).max(0.0) / 2.0
        } else {
            (pars.benefit.couple_25plus - uc_reduction).max(0.0) / 2.0
        }
    };

    let person = model.pop.alive_mut(p_id);
    if benefit > 0.0 {
        person.benefits.uc = true;
    }
    person.benefits.benefits += benefit;

    // extra for children
    let num_child_benefits = n_uc_deps.min(2) as f64;
    let benefit = pars.benefit.ea_children * num_child_benefits;
    person.benefits.benefits += benefit / (if person.kinship.is_single() { 1.0 } else { 2.0 });
    if benefit > 0.0 {
        person.benefits.uc = true;
    }

    // extra for disabled children
    let benefit = pars.benefit.ea_disabled_children[0] * n_mildly_disabled_uc_deps as f64
        + pars.benefit.ea_disabled_children[1] * n_crit_disabled_uc_deps as f64;
    person.benefits.benefits += benefit / (if person.kinship.is_single() { 1.0 } else { 2.0 });
    if benefit > 0.0 {
        person.benefits.uc = true;
    }

    // extra for disability
    if person.care.need_level > 1 {
        // limited capacity for work and related activity
        person.benefits.benefits += pars.benefit.lcfw_component;
        person.benefits.uc = true;
    }

    // extra for social work
    if person.care.social_work > 35 {
        person.benefits.benefits += pars.benefit.carers_component;
        person.benefits.uc = true;
    }
}

fn calc_pension_credit(
    person: &Person,
    model: &Model,
    pars: &ModelPars,
) -> Option<(Id, f64, bool)> {
    // condition 1: 65 or older (both if a couple)
    let retirement_age = Age::years(pars.work.age_retirement);
    if person.basic.age < retirement_age {
        return None;
    }
    let mut benefits = 0.0;
    let mut guarantee_credit = false;
    let maybe_partner = person.partner(model);
    let partner_retired = maybe_partner.is_some_and(|p| p.basic.age >= retirement_age);

    if let Some(partner) = maybe_partner
        && partner_retired
    {
        let tot_income = person.work.income + partner.work.income;
        let tot_wealth = person.work.financial_wealth + partner.work.financial_wealth;
        let aggregate_benefit_income = tot_income
            + ((tot_wealth - pars.benefit.wealth_allowance_pc)
                / pars.benefit.saving_income_rate_pc)
                .floor()
                .max(0.0);
        let b = (pars.benefit.couple_pc - aggregate_benefit_income).max(0.0) / 2.0;
        if b > 0.0 {
            benefits += b;
            guarantee_credit = true;
        }
    } else if maybe_partner.is_none() {
        let benefit_income = person.work.income
            + ((person.work.financial_wealth - pars.benefit.wealth_allowance_pc)
                / pars.benefit.saving_income_rate_pc)
                .floor()
                .max(0.0);
        let b = (pars.benefit.single_pc - benefit_income).max(0.0);
        if b > 0.0 {
            benefits += b;
            guarantee_credit = true;
        }
    }

    // severe disability extra
    if person.care.need_level > 2 {
        benefits += pars.benefit.disability_component_pc;
        guarantee_credit = true;
    }
    // social carers extra // FIXME: sometimes >, others >=?
    if person.care.social_work >= 35 {
        benefits += pars.benefit.caring_component_pc;
        guarantee_credit = true;
    }

    // other benefits are for people with dependents
    if person.dependency.is_dependent() {
        return Some((person.id(), benefits, guarantee_credit));
    }

    let house = person.house(model);
    let n_deps = n_dependents(house, model) as f64;
    if n_deps > 0.0 {
        let mut tot_child_benefit = pars.benefit.child_component_pc * n_deps;
        if partner_retired {
            tot_child_benefit /= 2.0;
        }
        benefits += tot_child_benefit;
        guarantee_credit = true;

        // mildly disabled children
        let n_mildly_disabled_deps = n_mildly_disabled_dependents(house, model) as f64;
        let mut tot_disable_child_benefit =
            pars.benefit.disabled_child_component[0] * n_mildly_disabled_deps;
        if partner_retired {
            tot_disable_child_benefit /= 2.0;
        }
        benefits += tot_disable_child_benefit;

        // critically disabled children
        let n_crit_disabled_deps = n_crit_disabled_dependents(house, model) as f64;
        let mut tot_disable_child_benefit =
            pars.benefit.disabled_child_component[1] * n_crit_disabled_deps;
        if partner_retired {
            tot_disable_child_benefit /= 2.0;
        }
        benefits += tot_disable_child_benefit;
    }

    if !house.income.owned_by_occupants {
        if let Some(partner) = maybe_partner
            && partner_retired
        {
            let tot_wealth = person.work.financial_wealth + partner.work.financial_wealth;
            if tot_wealth < pars.benefit.housing_benefit_wealth_threshold || guarantee_credit {
                let iroom = compute_max_rooms(house, model) - 1;
                let town = house.town(model);
                benefits += town.lha[iroom];
            }
        } else if maybe_partner.is_none() {
            if person.work.financial_wealth < pars.benefit.housing_benefit_wealth_threshold
                || guarantee_credit
            {
                let iroom = compute_max_rooms(house, model) - 1;
                let town = house.town(model);
                benefits += town.lha[iroom];
            }
        }
    }
    Some((person.id(), benefits, guarantee_credit))
}

fn pension_credit(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    let eligible: Vec<_> = model
        .pop
        .alives(order)
        .filter_map(|p| calc_pension_credit(p, model, pars))
        .collect();

    for (p_id, benefits, guarantee_credit) in eligible {
        let person = model.pop.alive_mut(p_id);
        person.benefits.benefits += benefits;
        person.benefits.guarantee_credit = guarantee_credit;
    }
}

fn compute_max_rooms(house: &House, model: &Model) -> usize {
    let mut allowed_rooms = 0;
    let mut n_male_teens = 0;
    let mut n_female_teens = 0;
    let mut n_children = 0;
    let mut n_couples = 0;

    for &o_id in house.basic.occupants() {
        let occ = model.pop.alive(o_id);
        if let Some(partner) = occ.partner(model) {
            if living_together(occ, partner) {
                n_couples += 1;
            }
        }

        if occ.basic.age >= Age::years(16) {
            allowed_rooms += 1
        } else if occ.basic.age >= Age::years(10) {
            match occ.basic.gender {
                Gender::Female => n_female_teens += 1,
                Gender::Male => n_male_teens += 1,
            }
        } else {
            n_children += 1;
        }
    }

    allowed_rooms += (n_male_teens + 1) / 2;
    allowed_rooms += (n_female_teens + 1) / 2;
    allowed_rooms += (n_children + 1) / 2;
    allowed_rooms -= n_couples / 2;

    allowed_rooms.min(4)
}
