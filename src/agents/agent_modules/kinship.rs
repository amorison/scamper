use identity_hash::IntSet;

use crate::{
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::AliveOrDead,
    utilities::int_set_with_cap,
};

#[derive(Debug)]
pub struct Partnership {
    with: Id,
    duration: u32,
}

impl Partnership {
    pub fn new(with: Id) -> Self {
        Self { with, duration: 0 }
    }

    pub fn with(&self) -> Id {
        self.with
    }

    pub fn step_one_month(&mut self) {
        self.duration += 1;
    }
}

#[derive(Default)]
pub struct Kinship {
    pub father: Option<Id>,
    pub mother: Option<Id>,
    pub partnership: Option<Partnership>,
    pub children: Vec<Id>,
}

impl Kinship {
    pub fn add_child(&mut self, id: Id) {
        self.children.push(id);
    }

    pub fn is_single(&self) -> bool {
        self.partnership.is_none()
    }

    pub fn partner(&self) -> Option<Id> {
        self.partnership.as_ref().map(|f| f.with)
    }

    pub fn parents(&self) -> Vec<Id> {
        let mut parents = Vec::with_capacity(2);
        if let Some(mother_id) = self.mother {
            parents.push(mother_id);
        }
        if let Some(father_id) = self.father {
            parents.push(father_id);
        }
        parents
    }

    pub fn n_children(&self) -> usize {
        self.children.len()
    }

    pub fn sibling_with(&self, other: &Kinship) -> bool {
        self.father.is_some() && self.father == other.father
            || self.mother.is_some() && self.mother == other.mother
    }

    pub fn parent_of(&self, id: Id) -> bool {
        self.children.contains(&id)
    }
}

pub fn are_siblings(p1: &Person, p2: &Person) -> bool {
    p1.kinship.sibling_with(&p2.kinship)
}

pub fn are_parent_child(p1: &Person, p2: &Person) -> bool {
    p1.kinship.parent_of(p2.id()) || p2.kinship.parent_of(p1.id())
}

/// Set of siblings (full and half, alive or dead).
pub fn siblings(p1: AliveOrDead, model: &Model) -> IntSet<Id> {
    let mut siblings = int_set_with_cap(10);

    for parent_id in p1.kinship().parents() {
        let parent = model.pop.get(parent_id);
        for child_id in parent.kinship().children.iter().copied() {
            if child_id != p1.id() {
                siblings.insert(child_id);
            }
        }
    }

    siblings
}
