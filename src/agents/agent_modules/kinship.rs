use std::collections::HashSet;

use crate::{
    full_model::{
        Model,
        person::{Id, Person},
    },
    population::AliveOrDead,
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
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

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

    pub fn full_sibling_with(&self, other: &Kinship) -> bool {
        self.father.is_some() && self.mother.is_some() && self.parents() == other.parents()
    }

    pub fn parent_of(&self, id: Id) -> bool {
        self.children.contains(&id)
    }
}

pub fn are_siblings(p1: &Person, p2: &Person) -> bool {
    p1.kinship.sibling_with(&p2.kinship)
}

pub fn are_full_siblings(p1: &Person, p2: &Person) -> bool {
    p1.kinship.full_sibling_with(&p2.kinship)
}

pub fn are_parent_child(p1: &Person, p2: &Person) -> bool {
    p1.kinship.parent_of(p2.id()) || p2.kinship.parent_of(p1.id())
}

// FIXME: this currently collect siblings that are alive or dead, check
// that this is what we need.
// FIXME: check if we use the separation between full and half siblings
/// Sets of full and half siblings.
pub fn siblings(p1: AliveOrDead, model: &Model) -> (HashSet<Id>, HashSet<Id>) {
    let mut full = HashSet::new();
    let mut half = HashSet::new();

    let mut parents = Vec::with_capacity(2);
    if let Some(father_id) = p1.kinship().father {
        parents.push(father_id);
    }
    if let Some(mother_id) = p1.kinship().mother {
        parents.push(mother_id);
    }

    for parent_id in parents {
        let parent = model.pop.get(parent_id);
        for child_id in parent.kinship().children.iter().copied() {
            let child = model.pop.get(child_id);
            if p1.kinship().sibling_with(child.kinship()) {
                if p1.kinship().full_sibling_with(child.kinship()) {
                    full.insert(child_id);
                } else {
                    half.insert(child_id);
                }
            }
        }
    }

    (full, half)
}
