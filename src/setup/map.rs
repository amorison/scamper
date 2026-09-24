use std::cmp::min;

use rand::RngExt;

use crate::{
    ModelPars,
    agents::towns::Town,
    full_model::{Model, house::House},
    utilities::Location,
};

pub fn create_towns(pars: &ModelPars) -> Vec<Town> {
    let nx = pars.map.nx();
    let ny = pars.map.ny();
    let mut towns: Vec<_> = (0..ny)
        .flat_map(|iy| {
            (0..nx).map(move |ix| {
                let loc = Location::new(ix, iy);
                let density = pars.map.pop_density[iy][ix];
                let lha = pars
                    .lha
                    .local_housing_allowances
                    .each_ref()
                    .map(|lha| lha[iy][ix]);
                Town::new(loc, density, lha)
            })
        })
        .collect();

    let ids: Vec<_> = towns.iter().map(Town::id).collect();

    for town in towns.iter_mut() {
        let (x, y) = town.loc.x_y();
        for iy in y.saturating_sub(1)..min(y + 2, ny) {
            for ix in x.saturating_sub(1)..min(x + 2, nx) {
                if ix == x && iy == y {
                    continue;
                }

                town.adjacent.push(ids[iy * nx + ix]);
            }
        }
    }

    towns
}

pub fn initialise_houses_in_town(model: &mut Model, init_pop: usize) {
    // FIXME: this heuristic works for the default map, should be generalised.
    let n_houses = (init_pop as f64 / 10.0).ceil() as usize;

    for town in model.towns.values_mut() {
        for _ in 0..n_houses {
            if model.rng.random_bool(0.6 * town.density) {
                let house = House::new(town.id());
                let h_id = house.id();
                model.houses.insert(h_id, house);
                town.houses.push(h_id);
            }
        }
    }
}
