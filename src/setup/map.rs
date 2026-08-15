use std::cmp::min;

use rand::RngExt;

use crate::{
    ModelPars,
    agents::{
        agent_modules::basic_house,
        towns::{Location, Town},
    },
    full_model::{Model, house::House},
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
    let grid_dim = (init_pop as f64 / 10.0).sqrt().ceil() as usize;

    model.town_size = grid_dim + 1; // FIXME: get rid of +1

    for town in model.towns.values_mut() {
        for hy in 0..grid_dim {
            for hx in 0..grid_dim {
                if model.rng.random_bool(0.6 * town.density) {
                    let h_loc = basic_house::Location::new(hx, hy);
                    let house = House::new(town.id(), h_loc);
                    let h_id = house.id();
                    model.houses.insert(h_id, house);
                    town.houses.push(h_id);
                }
            }
        }
    }
}
