use std::{
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
};

use identity_hash::IdentityHashable;

use crate::{
    agents::{
        agent_modules::{
            basic_house::{BasicHouse, Location},
            income_house::IncomeHouse,
        },
        towns::{IdTown, Town},
    },
    full_model::Model,
};

static ID_HOUSE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdHouse(u64);

impl IdHouse {
    fn new() -> IdHouse {
        IdHouse(ID_HOUSE.fetch_add(1, Ordering::Relaxed))
    }
}

impl Hash for IdHouse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IdentityHashable for IdHouse {}

pub struct House {
    id: IdHouse,
    pub basic: BasicHouse,
    pub income: IncomeHouse,
}

impl House {
    pub fn new(town: IdTown, location: Location) -> Self {
        Self {
            id: IdHouse::new(),
            basic: BasicHouse::new(town, location),
            income: IncomeHouse::default(),
        }
    }

    pub fn id(&self) -> IdHouse {
        self.id
    }

    pub fn town<'a>(&self, model: &'a Model) -> &'a Town {
        model.towns.get(&self.basic.town()).unwrap()
    }
}
