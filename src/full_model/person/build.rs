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
        house::IdHouse,
        person::{Id, Person},
    },
    utilities::Age,
};

pub struct PersonAwaitingHouse {
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
}

impl PersonAwaitingHouse {
    pub fn new(gender: Gender, age: Age) -> Self {
        Self {
            id: Id::new(),
            basic: BasicInfo::new(gender, age),
            kinship: Kinship::default(),
            maternity: Maternity::default(),
            work: Work::default(),
            care: Care::default(),
            class: Class::default(),
            benefits: Benefits::default(),
            dependency: Dependency::default(),
            task: TaskPerson::default(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn with_house(self, h_id: IdHouse) -> Person {
        Person {
            id: self.id,
            basic: self.basic,
            kinship: self.kinship,
            maternity: self.maternity,
            work: self.work,
            care: self.care,
            class: self.class,
            benefits: self.benefits,
            dependency: self.dependency,
            task: self.task,
            house: h_id,
        }
    }
}
