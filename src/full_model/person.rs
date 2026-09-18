pub(crate) mod build;

use std::{
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
};

use identity_hash::IdentityHashable;

use crate::{
    agents::agent_modules::{
        basic_info::{BasicInfo, Gender},
        benefits::Benefits,
        care::Care,
        class::Class,
        dependencies::Dependency,
        kinship::Kinship,
        maternity::Maternity,
        tasks::TaskPerson,
        work::Work,
    },
    full_model::{
        Model,
        house::{House, IdHouse},
        person::build::PersonAwaitingHouse,
    },
    utilities::{Age, HourInWeek},
};

static ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(u64);

impl Id {
    fn new() -> Id {
        Id(ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Hash for Id {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IdentityHashable for Id {}

/// Agent type.
pub struct Person {
    id: Id,
    pub basic: BasicInfo,
    pub kinship: Kinship,
    pub maternity: Maternity,
    pub work: Work,
    pub care: Care,
    pub class: Class,
    pub benefits: Benefits,
    pub dependency: Dependency,
    pub task: TaskPerson,
    pub house: IdHouse,
}

pub struct DeadPerson {
    id: Id,
    pub kinship: Kinship,
}

impl Person {
    pub fn baby(gender: Gender, house: IdHouse) -> Self {
        // FIXME: class is set later via `set_as_guardian_dependent`, maybe this can be abstracted
        // more neatly.
        PersonAwaitingHouse::new(gender, Age::new(), Class::new(0)).with_house(house)
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn partner<'a>(&self, model: &'a Model) -> Option<&'a Self> {
        self.kinship.partner().map(|id| model.pop.alive(id))
    }

    pub fn house<'a>(&self, model: &'a Model) -> &'a House {
        model.houses.get(&self.house).unwrap()
    }

    pub fn how_busy_at(&self, hour: HourInWeek) -> f64 {
        self.task.how_busy_at(hour)
    }

    pub fn is_female(&self) -> bool {
        matches!(self.basic.gender, Gender::Female)
    }

    pub fn is_male(&self) -> bool {
        matches!(self.basic.gender, Gender::Male)
    }
}

impl From<Person> for DeadPerson {
    fn from(value: Person) -> Self {
        let Person { id, kinship, .. } = value;
        Self { id, kinship }
    }
}

impl DeadPerson {
    pub fn id(&self) -> Id {
        self.id
    }
}
