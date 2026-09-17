use crate::N_CLASSES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(u32);

pub struct Class {
    pub rank: Rank,
    pub parent_rank: Rank,
}

impl Rank {
    pub fn new(rank: u32) -> Self {
        assert!((0..N_CLASSES as u32).contains(&rank));
        Self(rank)
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }

    pub fn increment(&mut self) {
        self.0 = (self.0 + 1).min(N_CLASSES as u32 - 1);
    }
}

impl Class {
    pub fn rank_idx(&self) -> usize {
        self.rank.index()
    }
}

impl Default for Class {
    fn default() -> Self {
        Self {
            rank: Rank::new(0),
            parent_rank: Rank::new(0),
        }
    }
}
