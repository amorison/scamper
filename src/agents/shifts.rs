use rand::{Rng, RngExt, seq::IteratorRandom};
use rand_distr::{Distribution, uniform::SampleRange, weighted::WeightedAliasIndex};

use crate::{
    ModelPars,
    utilities::{DayInWeek, HourInDay},
};

#[derive(Debug, Clone)]
pub struct Shift {
    pub days: Vec<DayInWeek>,
    pub shift_hours: Vec<HourInDay>,
    social_index: f64,
}

pub struct ShiftPool {
    shifts: Vec<Shift>,
    sampler: WeightedAliasIndex<f64>,
}

impl ShiftPool {
    pub fn new<R: Rng>(pars: &ModelPars, rng: &mut R) -> Self {
        // FIXME: what is this 9000?
        let f = 9e3 / pars.work.shifts_weights.iter().sum::<f64>();
        let mut all_hours = pars.work.shifts_weights.map(|w| (f * w).round() as u32);

        let mut sum_hours: u32 = all_hours.iter().sum();

        // FIXME: number of shifts should scale with population size
        let mut shifts = Vec::new();
        for _ in 0..1000 {
            let mut ih = 0;
            let mut i = (0..sum_hours).sample_single(rng).unwrap();
            while i > all_hours[ih] {
                i = i.saturating_sub(all_hours[ih]);
                ih += 1;
            }
            all_hours[ih] -= 1;
            sum_hours -= 1;

            let mut shift = vec![ih];

            // extend shift hours in both directions according to weight until 8 hours are reached or
            // weights on both sides are 0
            while shift.len() < 8 {
                // hours before and after ih with wraparound
                let next_hours = ((23 + ih) % 24, (shift.last().unwrap() + 1) % 24);
                let weights = (all_hours[next_hours.0], all_hours[next_hours.1]);
                let total = weights.0 + weights.1;
                if total == 0 {
                    break;
                }

                if (0..total).sample_single(rng).unwrap() < weights.0 {
                    shift.insert(0, next_hours.0);
                    all_hours[next_hours.0] -= 1;
                } else {
                    shift.push(next_hours.1);
                    all_hours[next_hours.1] -= 1;
                }
                sum_hours -= 1;
            }

            shifts.push(shift);
        }

        let mut all_shifts = Vec::with_capacity(shifts.len());

        for shift in shifts {
            let mut days = Vec::with_capacity(7);
            let mut we_soc_index = 0.0;
            if rng.random_bool(pars.work.prob_saturday_shift) {
                days.push(5);
                we_soc_index -= 1.0;
            }
            if rng.random_bool(pars.work.prob_sunday_shift) {
                days.push(6);
                we_soc_index -= 1.0 + pars.work.sunday_social_index;
            }
            if days.is_empty() {
                days = (0..5).collect();
            } else {
                days.extend((0..5).sample(rng, 5 - days.len()));
            }

            let social_index = (pars.work.shift_beta * pars.work.shifts_weights[shift[0]]
                + pars.work.day_beta * we_soc_index)
                .exp();

            all_shifts.push(Shift {
                days: days.into_iter().map(DayInWeek::new).collect(),
                shift_hours: shift
                    .into_iter()
                    .map(|h| HourInDay::new(h as u32))
                    .collect(),
                social_index,
            });
        }

        let weights = all_shifts.iter().map(|s| s.social_index).collect();
        let sampler = WeightedAliasIndex::new(weights).expect("error building shift sampler");

        Self {
            shifts: all_shifts,
            sampler,
        }
    }

    // TODO: draw without replacement?
    /// Sample `n` non-distinct shifts, weighted by their social index.
    pub fn sample<R: Rng>(&self, rng: &mut R, n: usize) -> Vec<Shift> {
        (&self.sampler)
            .sample_iter(rng)
            .take(n)
            .map(|i| &self.shifts[i])
            .cloned()
            .collect()
    }
}
