use rand::RngExt;

use crate::{
    ModelPars,
    agents::{
        agent_modules::class::Rank,
        interactions::{dependencies::resolve_dependency, family::resolve_partnership},
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    simulate::{
        move_house::{Proximity, move_people_to_empty_house},
        social::student_start_working,
    },
    utilities::{Date, calc_rate_bias, try_rand_yearly2monthly},
};

#[derive(Default)]
pub struct DivorceCache {
    class_bias: [f64; 5],
}

pub fn divorce_pre_calc(model: &mut Model, pars: &ModelPars) {
    model.divorce_cache.class_bias = calc_rate_bias(
        |i| model.social_cache.social_class_shares[i],
        pars.divorce.divorce_bias,
    );
}

fn divorce_probability(raw_rate: f64, rank: Rank, model: &Model) -> f64 {
    (raw_rate * model.divorce_cache.class_bias[rank.index()]).clamp(0.0, 1.0)
}

pub fn divorce(man_id: Id, date: Date, model: &mut Model, pars: &ModelPars) -> bool {
    let man = model.population.get(&man_id).unwrap();
    assert!(man.is_male());
    let wife_id = man.kinship.partner().expect("a single man cannot divorce");

    // This is here to manage the sweeping through of this parameter
    // but only for the years after 2012
    let idecade = (man.basic.age.year_month().0 / 10) as usize;
    let raw_rate = if date.year_month().0 < pars.divorce.the_present {
        pars.divorce.basic_divorce_rate * pars.divorce.divorce_modifier_by_decade[idecade]
    } else {
        pars.divorce.variable_divorce * pars.divorce.divorce_modifier_by_decade[idecade]
    };

    let divorce_prob = divorce_probability(raw_rate, man.class.rank, model);

    if try_rand_yearly2monthly(divorce_prob, &mut model.rng) {
        resolve_partnership(man_id, wife_id, model);

        let wife = model.population.get(&wife_id).unwrap();
        if wife.work.is_student() {
            student_start_working(wife_id, model, pars);
        }

        let man = model.population.get(&man_id).unwrap();
        let mut people_to_move = vec![man_id];
        let deps = man.dependency.dependents.clone();
        for child_id in deps {
            let child = model.population.get(&child_id).unwrap();
            assert!(child.basic.alive);
            let man_custody = child.kinship.father == Some(man_id)
                && (child.kinship.mother != Some(wife_id)
                    || model
                        .rng
                        .random_bool(pars.divorce.prob_children_with_father));
            if man_custody {
                people_to_move.push(child_id);
                resolve_dependency(wife_id, child_id, model);
            } else {
                resolve_dependency(man_id, child_id, model);
            }
        }

        // FIXME: maybe have this probability as a parameter?
        let prox = if model.rng.random_bool(0.5) {
            Proximity::Near
        } else {
            Proximity::Far
        };
        move_people_to_empty_house(&people_to_move, prox, model);

        true
    } else {
        false
    }
}

pub fn select_divorce(person: &Person) -> bool {
    person.basic.alive && person.is_male() && !person.kinship.is_single()
}
