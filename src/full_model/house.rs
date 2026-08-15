use std::sync::atomic::{AtomicU64, Ordering};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdHouse(u64);

impl IdHouse {
    fn new() -> IdHouse {
        IdHouse(ID_HOUSE.fetch_add(1, Ordering::Relaxed))
    }
}

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
