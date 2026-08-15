use std::sync::atomic::{AtomicU64, Ordering};

use crate::full_model::house::IdHouse;

static ID_TOWN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdTown(u64);

impl IdTown {
    fn new() -> IdTown {
        IdTown(ID_TOWN.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Location(usize, usize);

impl Location {
    pub fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    pub fn x_y(&self) -> (usize, usize) {
        (self.0, self.1)
    }

    pub fn manhattan_dist(&self, other: &Location) -> usize {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }

    pub fn is_adjacent(&self, other: &Location) -> bool {
        self.0.abs_diff(other.0) <= 1 && self.1.abs_diff(other.1) <= 1
    }
}

pub struct Town {
    id: IdTown,
    pub loc: Location,
    /// Relative population density.
    pub density: f64,
    pub houses: Vec<IdHouse>,
    pub adjacent: Vec<IdTown>,
    pub lha: [f64; 4],
}

impl Town {
    pub fn new(loc: Location, density: f64, lha: [f64; 4]) -> Self {
        Self {
            id: IdTown::new(),
            loc,
            density,
            houses: Vec::new(),
            adjacent: Vec::new(),
            lha,
        }
    }

    pub fn id(&self) -> IdTown {
        self.id
    }

    pub fn manhattan_dist(&self, other: &Town) -> usize {
        self.loc.manhattan_dist(&other.loc)
    }

    pub fn is_adjacent(&self, other: &Town) -> bool {
        self.loc.is_adjacent(&other.loc)
    }
}
