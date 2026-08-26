mod agents;
mod analysis;
pub mod cli;
mod common;
mod full_model;
mod population;
mod run_model;
mod setup;
mod simulate;
mod utilities;

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

pub const N_CLASSES: usize = 5;
pub const N_CARE_LEVELS: usize = 5;
pub const N_AGE_BANDS: usize = 6;

/// Map properties.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Map {
    pop_density: Vec<Vec<f64>>,
}

impl Default for Map {
    fn default() -> Self {
        let pop_density = vec![
            vec![0.0, 0.1, 0.2, 0.1, 0.0, 0.0, 0.0, 0.0],
            vec![0.1, 0.1, 0.2, 0.2, 0.3, 0.0, 0.0, 0.0],
            vec![0.0, 0.2, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.2, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0],
            vec![0.4, 0.0, 0.2, 0.2, 0.4, 0.0, 0.0, 0.0],
            vec![0.6, 0.0, 0.0, 0.3, 0.8, 0.2, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.6, 0.8, 0.4, 0.0, 0.0],
            vec![0.0, 0.0, 0.2, 1.0, 0.8, 0.6, 0.1, 0.0],
            vec![0.0, 0.0, 0.1, 0.2, 1.0, 0.6, 0.3, 0.4],
            vec![0.0, 0.0, 0.5, 0.7, 0.5, 1.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.2, 0.4, 0.6, 1.0, 1.0, 0.0],
            vec![0.0, 0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0],
        ];
        Self { pop_density }
    }
}

impl Map {
    pub fn nx(&self) -> usize {
        self.pop_density[0].len()
    }

    pub fn ny(&self) -> usize {
        self.pop_density.len()
    }
}

/// Housing benefits.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct BenefitMap {
    local_housing_allowances: [Vec<Vec<f64>>; 4],
}

impl Default for BenefitMap {
    fn default() -> Self {
        // FIXME: set at 0.0 in NI, is this OK?
        let local_housing_allowances = [
            vec![
                vec![0.0, 91.81, 91.81, 91.81, 0.0, 0.0, 0.0, 0.0],
                vec![91.81, 91.81, 91.81, 91.81, 97.81, 0.0, 0.0, 0.0],
                vec![0.0, 91.81, 91.81, 79.24, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 84.23, 94.82, 127.33, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 80.55, 72.00, 74.15, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 79.24, 90.90, 83.78, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 85.00, 100.05, 69.73, 0.0, 0.0],
                vec![0.0, 0.0, 71.41, 105.04, 94.80, 90.90, 90.64, 0.0],
                vec![0.0, 0.0, 65.59, 92.05, 101.84, 106.14, 133.72, 95.77],
                vec![0.0, 0.0, 103.56, 132.43, 163.67, 276.51, 165.05, 0.0],
                vec![0.0, 0.0, 116.52, 105.94, 120.03, 222.54, 170.83, 0.0],
                vec![0.0, 104.89, 96.98, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            vec![
                vec![0.0, 110.72, 110.72, 110.72, 0.0, 0.0, 0.0, 0.0],
                vec![110.72, 110.72, 110.72, 110.72, 133.48, 0.0, 0.0, 0.0],
                vec![0.0, 110.72, 110.72, 103.85, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 103.85, 120.03, 154.28, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 97.81, 92.05, 87.45, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 92.05, 103.56, 97.81, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 113.92, 122.36, 86.30, 0.0, 0.0],
                vec![0.0, 0.0, 91.43, 123.58, 107.11, 108.26, 115.58, 0.0],
                vec![0.0, 0.0, 86.00, 117.37, 127.62, 134.00, 153.79, 120.02],
                vec![0.0, 0.0, 126.92, 160.73, 192.48, 320.74, 204.35, 0.0],
                vec![0.0, 0.0, 141.24, 136.93, 161.07, 280.60, 210.17, 0.0],
                vec![0.0, 132.32, 122.36, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            vec![
                vec![0.0, 126.92, 126.92, 126.92, 0.0, 0.0, 0.0, 0.0],
                vec![126.92, 126.92, 126.92, 126.92, 172.60, 0.0, 0.0, 0.0],
                vec![0.0, 126.92, 126.92, 128.19, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 120.29, 137.31, 192.06, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 115.07, 109.62, 103.56, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 104.89, 115.07, 114.00, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 130.00, 149.59, 103.56, 0.0, 0.0],
                vec![0.0, 0.0, 110.41, 137.32, 116.53, 123.90, 133.35, 0.0],
                vec![0.0, 0.0, 101.11, 135.19, 135.96, 144.04, 178.71, 139.42],
                vec![0.0, 0.0, 150.00, 192.03, 230.14, 376.04, 257.16, 0.0],
                vec![0.0, 0.0, 164.79, 161.10, 190.02, 336.96, 257.16, 0.0],
                vec![0.0, 151.50, 145.43, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            vec![
                vec![0.0, 160.38, 160.38, 160.38, 0.0, 0.0, 0.0, 0.0],
                vec![160.38, 160.38, 160.38, 160.38, 228.99, 0.0, 0.0, 0.0],
                vec![0.0, 160.38, 160.38, 189.07, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 180.00, 212.21, 276.92, 0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 158.90, 142.61, 138.08, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 134.02, 149.59, 149.59, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 150.00, 195.62, 132.33, 0.0, 0.0],
                vec![0.0, 0.0, 133.32, 186.47, 156.00, 156.05, 168.05, 0.0],
                vec![0.0, 0.0, 126.58, 173.09, 173.41, 192.75, 238.38, 184.11],
                vec![0.0, 0.0, 190.38, 257.09, 299.18, 442.42, 331.81, 0.0],
                vec![0.0, 0.0, 218.63, 200.09, 242.40, 429.53, 322.15, 0.0],
                vec![0.0, 185.29, 182.45, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
        ];
        Self {
            local_housing_allowances,
        }
    }
}

/// Population setup and dynamics.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Population {
    start_year: u32,
    end_year: u32,
    init_size: u32,

    /// From 1921 census, a population of males to be randomly generated in the given age range
    init_male_prop: f64,
    start_prob_married: f64,
    start_prob_orphan: f64,

    base_die_prob: f64,
    baby_die_prob: f64,
    female_age_die_prob: f64,
    female_age_scaling: f64,
    female_mortality_bias: f64,
    male_age_die_prob: f64,
    male_age_scaling: f64,
    male_mortality_bias: f64,

    prob_classes: [f64; N_CLASSES],
}

impl Default for Population {
    fn default() -> Self {
        Self {
            start_year: 1920,
            end_year: 2040,
            init_size: 50_000,
            init_male_prop: 0.477,
            start_prob_married: 0.8,
            start_prob_orphan: 0.01,
            base_die_prob: 1e-4,
            baby_die_prob: 5e-3,
            female_age_die_prob: 1.9e-4,
            female_age_scaling: 15.5,
            female_mortality_bias: 0.85,
            male_age_die_prob: 2.1e-4,
            male_age_scaling: 14.0,
            male_mortality_bias: 0.8,
            prob_classes: [0.2, 0.35, 0.25, 0.15, 0.05],
        }
    }
}

/// Reproduction parameters.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Birth {
    fertility_bias: f64,
    prev_child_fertility_bias: f64,
    min_pregnancy_age: u32,
    max_pregnancy_age: u32,
}

impl Default for Birth {
    fn default() -> Self {
        Self {
            fertility_bias: 0.9,
            prev_child_fertility_bias: 0.9,
            min_pregnancy_age: 16,
            max_pregnancy_age: 50,
        }
    }
}

/// Work and education.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Work {
    maternity_leave_income_reduction: f64, // FIXME: used but not present in Julia model! Also misnamed
    maternity_leave_duration: u32,
    min_statutory_maternity_pay: f64,
    age_teenagers: u32,
    age_adulthood: u32,
    age_independence: u32,
    age_retirement: u32,
    min_contribution_period: u32,
    wage_var: f64,
    income_initial_levels: [f64; N_CLASSES],
    final_income_mu: [f64; N_CLASSES],
    final_income_sigma: [f64; N_CLASSES],
    income_growth_rate: [f64; N_CLASSES],
    /// Age at which people can stop studying and start working.
    start_working_age: [u32; N_CLASSES],
    /// Working hours by care requirements.
    weekly_hours: [u32; N_CARE_LEVELS],
    constant_income: f64,
    constant_education: f64,
    edu_wage_sensitivity: f64,
    edu_rank_sensitivity: f64,
    care_education: f64,
    work_discounting_time: f64,
    move_out_prob: f64,
    tax_brackets: [f64; 3],   // FIXME: why 3?
    taxation_rates: [f64; 3], // FIXME: why 3?
    unemployment_age_bias: [f64; N_AGE_BANDS],
    unemployment_class_bias: f64,
    share_financial_wealth: f64,
    pension_return_rate: f64,
    shifts_weights: [f64; 24],
    prob_saturday_shift: f64,
    prob_sunday_shift: f64,
    sunday_social_index: f64,
    shift_beta: f64,
    day_beta: f64,
    probation_period: u32, // FIXME: unused
    hire_rate: f64,
}

impl Default for Work {
    fn default() -> Self {
        Self {
            maternity_leave_income_reduction: 0.9, // FIXME: used but not present in Julia model!
            maternity_leave_duration: 9,
            min_statutory_maternity_pay: 151.97,
            age_teenagers: 13,
            age_adulthood: 16,
            age_independence: 18,
            age_retirement: 65,
            min_contribution_period: 12 * 35,
            wage_var: 0.2,
            income_initial_levels: [6.0, 8.0, 10.0, 12.0, 15.0],
            final_income_mu: [2.5, 2.8, 3.2, 3.7, 4.5],
            final_income_sigma: [0.25, 0.3, 0.35, 0.4, 0.5],
            income_growth_rate: [0.4 / 12.0, 0.35 / 12.0, 0.3 / 12.0, 0.25 / 12.0, 0.2 / 12.0],
            start_working_age: [16, 18, 20, 22, 24],
            weekly_hours: [40, 20, 10, 0, 0],
            constant_income: 50.0,
            constant_education: 4.0,
            edu_wage_sensitivity: 0.1,
            edu_rank_sensitivity: 4.0,
            care_education: 0.0,
            work_discounting_time: 1.0,
            move_out_prob: 0.1,
            tax_brackets: [663.0, 228.0, 0.0],
            taxation_rates: [0.4, 0.2, 0.0],
            unemployment_age_bias: [1.0, 0.55, 0.35, 0.25, 0.2, 0.2],
            unemployment_class_bias: 0.75,
            share_financial_wealth: 0.3,
            pension_return_rate: 0.05 / 12.0,
            shifts_weights: [
                51.80, 66.10, 70.10, 71.40, 54.10, 63.40, 68.60, 65.00, 54.70, 35.00, 20.70, 15.70,
                13.00, 11.50, 9.10, 6.80, 4.60, 3.80, 3.20, 3.00, 4.60, 6.70, 13.90, 28.80,
            ],
            prob_saturday_shift: 0.2,
            prob_sunday_shift: 0.1,
            sunday_social_index: 0.5,
            shift_beta: 0.1,
            day_beta: 0.1,
            probation_period: 3,
            hire_rate: (4.0f64 / 3.0).ln(), // expected mean unemployment time of 4 months
        }
    }
}

/// Benefits.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Benefits {
    child_benefit_income_threshold: f64,
    first_child_benefit: f64,
    other_children_benefit: f64,

    care_dla: [f64; 3],     // FIXME: why 3?
    mobility_dla: [f64; 2], // FIXME: why 2?
    care_pip: [f64; 2],     // FIXME: why 2?
    mobility_pip: [f64; 2], // FIXME: why 2?
    care_aa: [f64; 2],      // FIXME: why 2?
    carers_allowance: f64,

    capital_high_threshold: f64,
    capital_low_threshold: f64,
    capital_income: f64,
    saving_uc_rate: f64,
    work_allowance_hs: f64,
    work_allowance_no_hs: f64,
    income_reduction: f64,
    single_below25: f64,
    single_25plus: f64,
    couple_below25: f64,
    couple_25plus: f64,
    ea_children: f64,
    ea_disabled_children: [f64; 2], // FIXME: why 2?
    lcfw_component: f64,
    carers_component: f64,

    single_pc: f64,
    couple_pc: f64,
    wealth_allowance_pc: f64,
    saving_income_rate_pc: f64,
    disability_component_pc: f64,
    caring_component_pc: f64,
    child_component_pc: f64,
    disabled_child_component: [f64; 2], // FIXME: why 2?
    housing_benefit_wealth_threshold: f64,
}

impl Default for Benefits {
    fn default() -> Self {
        Self {
            child_benefit_income_threshold: 5e4,
            first_child_benefit: 21.15,
            other_children_benefit: 14.0,
            care_dla: [23.7, 60.0, 89.6],
            mobility_dla: [23.7, 62.55],
            care_pip: [60.0, 89.6],
            mobility_pip: [23.7, 62.55],
            care_aa: [60.0, 89.6],
            carers_allowance: 67.6,
            capital_high_threshold: 1.6e4,
            capital_low_threshold: 6e3,
            capital_income: 4.35,
            saving_uc_rate: 250.0,
            work_allowance_hs: (293.0 * 12.0) / 52.0,
            work_allowance_no_hs: (515.0 * 12.0) / 52.0,
            income_reduction: 0.63,
            single_below25: (257.33 * 12.0) / 52.0,
            single_25plus: (324.84 * 12.0) / 52.0,
            couple_below25: ((403.93 * 12.0) / 52.0) / 2.0,
            couple_25plus: ((509.91 * 12.0) / 52.0) / 2.0,
            ea_children: (237.08 * 12.0) / 52.0,
            ea_disabled_children: [(128.89 * 12.0) / 52.0, (402.41 * 12.0) / 52.0],
            lcfw_component: (343.63 * 12.0) / 52.0,
            carers_component: (163.73 * 12.0) / 52.0,
            single_pc: 177.10, // top-up
            couple_pc: 270.30, // top-up
            wealth_allowance_pc: 1e4,
            saving_income_rate_pc: 5e2,
            disability_component_pc: 67.3,
            caring_component_pc: 37.7,
            child_component_pc: 54.6,
            disabled_child_component: [29.66, 92.54],
            housing_benefit_wealth_threshold: 1.6e4,
        }
    }
}

/// Divorce parameters.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Divorce {
    basic_divorce_rate: f64,
    divorce_modifier_by_decade: Vec<f64>,
    prob_children_with_father: f64,
    the_present: u32,
    variable_divorce: f64,
    divorce_bias: f64,
}

impl Default for Divorce {
    fn default() -> Self {
        Self {
            basic_divorce_rate: 0.06,
            divorce_modifier_by_decade: vec![
                0.0, 1.0, 0.9, 0.5, 0.4, 0.2, 0.1, 0.03, 0.01, 0.001, 0.001, 0.001, 0.0, 0.0, 0.0,
                0.0,
            ],
            prob_children_with_father: 0.1,
            the_present: 2012,
            variable_divorce: 0.06,
            divorce_bias: 0.9,
        }
    }
}

/// Divorce parameters.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Marriage {
    basic_male_marriage_prob: f64,
    male_marriage_modifier_by_decade: Vec<f64>,
    not_working_marriage_bias: f64,
    man_with_children_bias: f64,
    prob_apart_will_move_together: f64,
    couples_move_to_existing_household: f64,
    /// Effect of distance on marriage probability.
    beta_geo_exp: f64,
    student_factor: f64,
    /// Effect of class difference on marriage probability.
    beta_soc_exp: f64,
    rank_gender_bias: f64,
    /// Probability distribution of age difference.
    mode_age_diff: f64,
    male_older_factor: f64,
    male_younger_factor: f64,
    brides_children_exp: f64,
}

impl Default for Marriage {
    fn default() -> Self {
        Self {
            basic_male_marriage_prob: 0.7,
            male_marriage_modifier_by_decade: vec![
                0.0, 0.16, 0.5, 1.0, 0.8, 0.7, 0.66, 0.5, 0.4, 0.2, 0.1, 0.05, 0.01, 0.0, 0.0, 0.0,
            ],
            not_working_marriage_bias: 0.5,
            man_with_children_bias: 0.9,
            prob_apart_will_move_together: 1.0,
            couples_move_to_existing_household: 0.0,
            beta_geo_exp: 0.2,
            student_factor: 0.5,
            beta_soc_exp: 2.0,
            rank_gender_bias: 0.5,
            mode_age_diff: 2.0,
            male_older_factor: 0.1,
            male_younger_factor: 0.3,
            brides_children_exp: 0.5,
        }
    }
}

/// Social care.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Care {
    care_bias: f64,
    care_need_bias: f64,
    female_age_care_scaling: f64,
    male_age_care_scaling: f64,
    person_care_prob: f64,
    base_care_prob: f64,
    care_transition_rate: f64,
    child_care_demand: u32,
    /// Weekly care supply for child, teen, student, fixed shift, flexible shift, retired, unemployed
    care_supply_by_status: [u32; 7],
}

impl Default for Care {
    fn default() -> Self {
        Self {
            care_bias: 0.9,
            care_need_bias: 0.9,
            female_age_care_scaling: 19.0,
            male_age_care_scaling: 18.0,
            person_care_prob: 8e-4,
            base_care_prob: 2e-4,
            care_transition_rate: 0.7,
            child_care_demand: 168,
            care_supply_by_status: [0, 10, 24, 32, 32, 60, 48],
        }
    }
}

/// Care tasks parameters.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct TaskCare {
    /// How often to iterate care distribution.
    n_iter_care_dist: u32,
    stop_baby_care_age: u32,
    stop_child_care_age: u32,
    baby_care_per_day: u32,
    child_care_per_day: u32,
    social_care_demand_per_day: [u32; N_CARE_LEVELS],
    care_supply_maternity: u32,
    /// Effect of task importance on acceptance probability.
    accept_prob_polarity: f64,
    /// Care weight by relatedness and type. Relatedness (carer is): child, parent, partner,
    /// sibling, other. Type: see task kind.
    care_weight_related: [[f64; 3]; 5],
    /// Care weight by spatial distance (same house, same town, other).
    care_weight_distance: [f64; 3],
}

impl Default for TaskCare {
    fn default() -> Self {
        Self {
            n_iter_care_dist: 3,
            stop_baby_care_age: 1,
            stop_child_care_age: 13,
            baby_care_per_day: 14,
            child_care_per_day: 14,
            social_care_demand_per_day: [0, 2, 4, 8, 12],
            care_supply_maternity: 98,
            accept_prob_polarity: 2.0,
            care_weight_related: [
                [1e6, 0.5, 50.0],
                [1.0, 1.0, 50.0],
                [1e6, 1.0, 50.0],
                [0.8, 0.5, 50.0],
                [0.5, 0.2, 50.0],
            ],
            care_weight_distance: [1.0, 0.5, 0.1],
        }
    }
}

/// Housing parameters.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Housing {
    ownership_prob_exp: f64,
    income_ownership_shares: [f64; 10],
    age_ownership_shares: Vec<f64>, // FIXME: related to number of ho_age_ranges
    ho_age_ranges: Vec<f64>,
    ho_age_biases: Vec<f64>,
}

impl Default for Housing {
    fn default() -> Self {
        Self {
            ownership_prob_exp: 0.1,
            income_ownership_shares: [0.2, 0.4, 0.5, 0.57, 0.63, 0.67, 0.71, 0.75, 0.79, 0.83],
            age_ownership_shares: vec![0.12, 0.44, 0.61, 0.71, 0.77, 0.79],
            ho_age_ranges: vec![24.0, 34.0, 44.0, 54.0, 64.0], // FIXME: this needs to be sorted
            ho_age_biases: vec![1.0, 3.67, 5.0, 5.9, 6.4, 6.6],
        }
    }
}

/// Data files.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct DataFiles {
    ini_age: String,
    pre51_fertility: String,
    fertility: String,
    pre51_deaths: String,
    death_female: String,
    death_male: String,
    unemployment_rate: String,
    wealth_distribution: String,
}

impl Default for DataFiles {
    fn default() -> Self {
        Self {
            ini_age: "data/age_pyramid_1921_5y.csv".to_owned(),
            pre51_fertility: "data/birthrate_early.csv".to_owned(),
            fertility: "data/babyrate.txt.csv".to_owned(),
            pre51_deaths: "data/deathrates_early.csv".to_owned(),
            death_female: "data/deathrate.fem.csv".to_owned(),
            death_male: "data/deathrate.male.csv".to_owned(),
            unemployment_rate: "data/unemploymentrate.csv".to_owned(),
            wealth_distribution: "data/wealthDistribution.csv".to_owned(),
        }
    }
}

/// Simulation setup.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Simulation {
    /// Step size.
    dt: u32,
    seed: u64,
    /// Whether to print significant intermediate info.
    verbose: bool,
    /// How long simulation is suspended after printing info.
    sleep_time: f64,
    log_file: String,
    start_log_time: u32,
    end_log_time: u32,

    dump_agents: bool,
    dump_houses: bool,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            dt: 1,
            seed: 42,
            verbose: false,
            sleep_time: 0.0,
            log_file: "log.tsv".to_owned(),
            start_log_time: 0,
            end_log_time: 10_000,
            dump_agents: false,
            dump_houses: false,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct ModelPars {
    map: Map,
    lha: BenefitMap,
    population: Population,
    birth: Birth,
    work: Work,
    benefit: Benefits,
    divorce: Divorce,
    marriage: Marriage,
    care: Care,
    task_care: TaskCare,
    housing: Housing,
    data_files: DataFiles,
    simulation: Simulation,
}

impl ModelPars {
    fn read_from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read(path)?;
        match toml::from_slice(&content) {
            Ok(conf) => Ok(conf),
            Err(err) => {
                eprintln!("{err}");
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid config file",
                ))
            }
        }
    }

    fn save_to<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let content = toml::to_string(self).expect("failed to serialize parameters");
        fs::write(path, content)
    }
}
