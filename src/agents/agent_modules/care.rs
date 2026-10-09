use crate::ModelPars;

#[derive(Default)]
pub struct Care {
    pub need_level: u32,
    pub social_work: u32,
    pub child_work: u32,
    pub wealth_spent_on_care: f64,
    unmet: UnmetCare,
}

impl Care {
    pub fn index(&self) -> usize {
        self.need_level as usize
    }

    pub fn register_unmet_need(&mut self, unmet_care: f64, pars: &ModelPars) {
        self.unmet.accumulate(unmet_care, self.need_level, pars);
    }

    pub fn current_unmet_need(&self) -> f64 {
        if self.need_level == 0 {
            1.0
        } else {
            self.unmet.average_unmet_need()
        }
    }
}

#[derive(Default)]
struct UnmetCare {
    total: f64, // FIXME: don't need this?
    share: f64,
    times: f64,
}

impl UnmetCare {
    fn accumulate(&mut self, unmet_care: f64, level: u32, pars: &ModelPars) {
        if level == 0 {
            return;
        }
        self.total = self.total * pars.care.unmet_need_discount + unmet_care;
        let care_need = pars.task_care.social_care_demand_per_day[level as usize] as f64;
        self.share = self.share * pars.care.unmet_need_share_discount + unmet_care / care_need;
        self.times = self.times * pars.care.unmet_need_share_discount + 1.0;
    }

    fn average_unmet_need(&self) -> f64 {
        self.share / self.times
    }
}
