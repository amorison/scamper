use crate::full_model::person::Id;

#[derive(Default)]
pub struct Dependency {
    pub guardians: Vec<Id>,
    pub dependents: Vec<Id>,
    pub provider: Option<Id>,
    pub providees: Vec<Id>,
    pub past_guardians: Vec<Id>,
}

impl Dependency {
    pub fn is_dependent(&self) -> bool {
        !self.guardians.is_empty()
    }

    pub fn has_dependents(&self) -> bool {
        !self.dependents.is_empty()
    }
}
