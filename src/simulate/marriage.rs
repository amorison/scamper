use std::mem;

use rand::{RngExt, seq::IndexedRandom};

use crate::{
    ModelPars, N_CLASSES,
    agents::interactions::{
        dependencies::set_as_guardian_dependent,
        family::{related_1st_degree, set_as_partners},
        housing::living_together,
    },
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::PopIterOrder,
    simulate::move_house::{Proximity, move_people_to_empty_house, move_people_to_house},
    utilities::{Age, try_rand_yearly2monthly},
};

pub fn age_class(person: &Person) -> usize {
    person.basic.age.year_month().0 as usize / 10
}

#[derive(Default)]
pub struct MarriageCache {
    share_men_no_children: Vec<f64>,
    eligible_women: Vec<Id>,
}

pub fn marriage_pre_calc(model: &mut Model, order: &PopIterOrder, pars: &ModelPars) {
    model.marriage_cache.share_men_no_children.resize(20, 0.0);
    model.marriage_cache.share_men_no_children.fill(0.0);

    let mut n_all = [0.0; 20];
    for person in model.pop.alives(order).filter(|p| p.is_male()) {
        let ac = age_class(person);
        n_all[ac] += 1.0;
        // FIXME: only looks at dependent persons (which usually are underage and living in the same
        // household
        if !person.dependency.has_dependents() {
            model.marriage_cache.share_men_no_children[ac] += 1.0;
        }
    }
    model
        .marriage_cache
        .share_men_no_children
        .iter_mut()
        .enumerate()
        .for_each(|(ac, val)| *val /= n_all[ac]);

    model.marriage_cache.eligible_women.clear();
    model
        .pop
        .alives(order)
        .filter(|p| {
            p.is_female()
                && p.kinship.is_single()
                && p.basic.age > Age::years(pars.birth.min_pregnancy_age)
        })
        .for_each(|w| model.marriage_cache.eligible_women.push(w.id()));
}

fn age_factor(agem: Age, agef: Age, pars: &ModelPars) -> f64 {
    let diff = agem.years_f64() - agef.years_f64() - pars.marriage.mode_age_diff;
    if diff > 0.0 {
        1.0 / (pars.marriage.male_older_factor * diff.powi(2)).exp()
    } else {
        1.0 / (pars.marriage.male_younger_factor * diff.powi(2)).exp()
    }
}

fn marry_weight(man_id: Id, woman_id: Id, model: &Model, pars: &ModelPars) -> f64 {
    let man = model.pop.alive(man_id);
    let woman = model.pop.alive(woman_id);
    if living_together(man, woman) || related_1st_degree(man, woman) {
        return 0.0;
    }

    let geo_factor =
        1.0 / (pars.marriage.beta_geo_exp * geo_distance(man, woman, model, pars)).exp();

    let (student_factor, woman_rank) = if woman.work.is_student() {
        (pars.marriage.student_factor, woman.class.parent_rank)
    } else {
        (1.0, woman.class.rank)
    };

    let man_rank = man.class.rank_idx();
    let woman_rank = woman_rank.index();
    let status_distance = man_rank.abs_diff(woman_rank) as f64 / (N_CLASSES - 1) as f64;

    let beta_exponent = pars.marriage.beta_soc_exp
        * if man_rank < woman_rank {
            1.0
        } else {
            pars.marriage.rank_gender_bias
        };

    let soc_factor = 1.0 / (beta_exponent * status_distance).exp();

    let num_children_with_woman = woman.dependency.dependents.len() as f64;
    let children_factor = 1.0 / (pars.marriage.brides_children_exp * num_children_with_woman).exp();

    geo_factor
        * soc_factor
        * age_factor(man.basic.age, woman.basic.age, pars)
        * children_factor
        * student_factor
}

fn geo_distance(man: &Person, woman: &Person, model: &Model, pars: &ModelPars) -> f64 {
    let t_man = man.house(model).town(model);
    let t_woman = woman.house(model).town(model);
    let mdist = t_man.manhattan_dist(t_woman) as f64;
    let scale = (pars.map.nx() + pars.map.ny()) as f64;
    mdist / scale
}

pub fn select_marriage(person: &Person, pars: &ModelPars) -> bool {
    // FIXME: should it be age of adulthood (16), or 18?
    // If 16, shouldn't a married person in 16-18 range become independent from their parents?
    person.is_male()
        && person.kinship.is_single()
        && person.basic.age > Age::years(pars.work.age_adulthood)
        && person.care.need_level < 4
}

pub fn marriage(man_id: Id, model: &mut Model, pars: &ModelPars) {
    let man = model.pop.alive(man_id);

    let age_class = age_class(man);
    let mut man_marriage_prob = if age_class >= pars.marriage.male_marriage_modifier_by_decade.len()
    {
        0.0
    } else {
        pars.marriage.basic_male_marriage_prob
            * pars.marriage.male_marriage_modifier_by_decade[age_class]
    };

    if !man.work.is_worker() || man.care.need_level > 1 {
        man_marriage_prob *= pars.marriage.not_working_marriage_bias;
    }

    let snc = model.marriage_cache.share_men_no_children[age_class];
    let den = snc + (1.0 - snc) * pars.marriage.man_with_children_bias;

    let prob = man_marriage_prob / den
        * if man.dependency.has_dependents() {
            pars.marriage.man_with_children_bias
        } else {
            1.0
        };
    let prob = prob.clamp(0.0, 1.0);

    if !try_rand_yearly2monthly(prob, &mut model.rng) {
        return;
    }

    // temporarily take the list of eligible women out of the cache
    // for manipulation
    let mut women_ids = mem::take(&mut model.marriage_cache.eligible_women);
    if women_ids.is_empty() {
        return;
    }

    let n_women = women_ids.len();
    let mut weights = vec![0.0; n_women];
    let mut sum = 0.0;
    for (i, &w_id) in women_ids.iter().enumerate() {
        let w = marry_weight(man_id, w_id, model, pars);
        weights[i] = w;
        sum += w;
    }
    if sum == 0.0 {
        return;
    }

    let indices: Vec<_> = (0..n_women).collect();
    let selected = indices
        .choose_weighted(&mut model.rng, |&i| weights[i])
        .unwrap();
    let woman_id = women_ids.swap_remove(*selected);

    set_as_partners(man_id, woman_id, model);
    join_couple(man_id, woman_id, model, pars);

    // dependents become joint dependents
    let man = model.pop.alive(man_id);
    let dep_man = man.dependency.dependents.clone();
    let woman = model.pop.alive(woman_id);
    let dep_woman = woman.dependency.dependents.clone();
    for child_id in dep_man {
        set_as_guardian_dependent(woman_id, child_id, model);
    }
    for child_id in dep_woman {
        set_as_guardian_dependent(man_id, child_id, model);
    }

    // put the list back in the cache
    mem::swap(&mut model.marriage_cache.eligible_women, &mut women_ids);
}

fn gather_dependents_single(person: &Person, model: &Model) -> Vec<Id> {
    // for now simply all dependents who already live with person
    person
        .dependency
        .dependents
        .iter()
        .copied()
        .filter(|&dep_id| {
            let dep = model.pop.alive(dep_id);
            assert_eq!(dep.dependency.guardians, vec![person.id()]);
            living_together(person, dep)
        })
        .collect()
}

fn join_couple(man_id: Id, woman_id: Id, model: &mut Model, pars: &ModelPars) -> bool {
    if !model
        .rng
        .random_bool(pars.marriage.prob_apart_will_move_together)
    {
        // they stay apart
        return false;
    }

    // decide who leads the move
    let mut people_to_move = if model.rng.random_bool(0.5) {
        vec![man_id, woman_id]
    } else {
        vec![woman_id, man_id]
    };

    let man = model.pop.alive(man_id);
    let woman = model.pop.alive(woman_id);
    people_to_move.extend(gather_dependents_single(man, model));
    people_to_move.extend(gather_dependents_single(woman, model));

    if model
        .rng
        .random_bool(pars.marriage.couples_move_to_existing_household)
    {
        let n_occ_man = man.house(model).basic.occupants().len();
        let n_occ_woman = woman.house(model).basic.occupants().len();
        // FIXME: this minimises occupancy of the target house, but not necessarily the number of
        // people to move
        let target_house = if n_occ_man > n_occ_woman {
            woman.house
        } else {
            man.house
        };
        move_people_to_house(&people_to_move, target_house, model);
    } else {
        let prox = if model.rng.random_bool(0.5) {
            Proximity::Here
        } else {
            Proximity::Near
        };
        move_people_to_empty_house(&people_to_move, prox, model);
    }

    true
}
