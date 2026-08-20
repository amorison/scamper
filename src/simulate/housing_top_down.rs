use rand::seq::IndexedRandom;

use crate::{ModelPars, full_model::Model};

pub fn house_ownership(model: &mut Model, pars: &ModelPars) {
    let mut households: Vec<_> = model
        .houses
        .values_mut()
        .filter(|h| h.basic.is_occupied())
        .collect();
    households.sort_unstable_by(|h1, h2| {
        h1.income
            .household_income
            .partial_cmp(&h2.income.household_income)
            .unwrap()
    });
    let hh_ids: Vec<_> = households.iter().map(|h| h.id()).collect();

    for (i, h) in households.into_iter().enumerate() {
        h.income.income_decile = (10 * i / hh_ids.len()) as u32;

        // TODO: (in Julia) this might need rethinking. What about shared flats, adult and earning
        // children, grandparents living in the house, etc?
        let mut n = 0.0;
        let mut age = 0.0;

        for &p_id in h.basic.occupants() {
            let agent = model.pop.alive(p_id);
            if !agent.dependency.has_guardians() {
                n += 1.0;
                age += agent.basic.age.years_f64();
            }
        }
        h.income.age_occupants = age / n;
    }

    let mut houses_by_age = vec![vec![]; pars.housing.age_ownership_shares.len()];

    for d in 0..10 {
        houses_by_age.iter_mut().for_each(|hba| hba.clear());

        // sort households in decile d by age class
        for &h_id in &hh_ids {
            let house = model.houses.get(&h_id).unwrap();
            if house.income.income_decile == d {
                let i_age_range = pars
                    .housing
                    .ho_age_ranges
                    .partition_point(|&a| house.income.age_occupants < a);
                houses_by_age[i_age_range].push(h_id);
            }
        }

        // FIXME: this seems very prescriptive
        for (a, age_house) in houses_by_age.iter().enumerate() {
            // how many people should own their house in this age class
            let share = pars.housing.age_ownership_shares[a];
            let n_empirical_owners = (share * age_house.len() as f64).floor() as usize;
            let (owned_houses, rented_houses): (Vec<_>, Vec<_>) = age_house
                .iter()
                .copied()
                .partition(|h_id| model.houses.get(h_id).unwrap().income.owned_by_occupants);
            if n_empirical_owners < owned_houses.len() {
                let num_houses_to_sell = owned_houses.len() - n_empirical_owners;
                // TODO: add ownership index
                // And then houses to sell are weighted by 1/exp(pars.housing.ownership_prob_exp *
                // house.ownership_index) for sampling
                for h_to_sell in owned_houses.sample(&mut model.rng, num_houses_to_sell) {
                    let house = model.houses.get_mut(h_to_sell).unwrap();
                    house.income.owned_by_occupants = false;
                }
            } else if n_empirical_owners > owned_houses.len() {
                let num_houses_to_buy = n_empirical_owners - owned_houses.len();
                // TODO: add ownership index, houses to buy are weighted by
                // pars.housing.ownership_prob_exp * house.ownership_index
                for h_to_buy in rented_houses.sample(&mut model.rng, num_houses_to_buy) {
                    let house = model.houses.get_mut(h_to_buy).unwrap();
                    house.income.owned_by_occupants = true;
                }
            }
        }
    }
}
