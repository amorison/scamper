use crate::{ModelPars, N_CLASSES};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Rank(u32);

#[derive(Clone, Copy)]
pub struct Class {
    rank: Rank,
    parent_rank: Rank,
    education: Rank,
}

impl Rank {
    fn new(rank: u32) -> Self {
        assert!((0..N_CLASSES as u32).contains(&rank));
        Self(rank)
    }

    fn index(&self) -> usize {
        self.0 as usize
    }

    fn increment(&mut self) {
        self.0 = (self.0 + 1).min(N_CLASSES as u32 - 1);
    }
}

impl Class {
    pub fn new(rank: u32) -> Self {
        Self {
            rank: Rank::new(rank),
            parent_rank: Rank::new(0),
            education: Rank::new(0),
        }
    }

    pub fn rank_idx(&self) -> usize {
        self.rank.index()
    }

    pub fn set_from_parent_class(&mut self, parent_class: Class) {
        self.parent_rank = self.parent_rank.max(parent_class.rank);
        self.rank = self.parent_rank;
    }

    pub fn study(&mut self) {
        self.education.increment();
    }

    pub fn has_max_education(&self) -> bool {
        self.education.0 >= N_CLASSES as u32 - 1
    }

    /// Influence of education gap with parents on probability of studying.
    pub fn edu_gap_factor(&self, pars: &ModelPars) -> f64 {
        let de = self.parent_rank.0 as f64 - self.education.0 as f64;
        let exp_edu = (pars.work.edu_rank_sensitivity * de).exp();
        exp_edu / (exp_edu + pars.work.constant_education)
    }

    pub fn enter_workforce(&mut self) {
        self.rank = self.education;
    }
}
