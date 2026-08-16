use std::collections::HashMap;

use rand::{Rng, seq::SliceRandom};

use crate::{
    agents::agent_modules::kinship::Kinship,
    full_model::person::{DeadPerson, Id, Person},
};

#[derive(Clone, Copy)]
pub enum AliveOrDead<'a> {
    Alive(&'a Person),
    Dead(&'a DeadPerson),
}

impl<'a> AliveOrDead<'a> {
    pub fn id(self) -> Id {
        match self {
            AliveOrDead::Alive(person) => person.id(),
            AliveOrDead::Dead(dead_person) => dead_person.id(),
        }
    }

    pub fn kinship(self) -> &'a Kinship {
        match self {
            AliveOrDead::Alive(person) => &person.kinship,
            AliveOrDead::Dead(dead_person) => &dead_person.kinship,
        }
    }

    pub fn is_alive(self) -> bool {
        match self {
            AliveOrDead::Alive(_) => true,
            AliveOrDead::Dead(_) => false,
        }
    }

    pub fn is_alive_and<F: FnOnce(&Person) -> bool>(self, predicate: F) -> bool {
        match self {
            AliveOrDead::Alive(person) => predicate(person),
            AliveOrDead::Dead(_) => false,
        }
    }
}

pub struct Population {
    living: HashMap<Id, Person>,
    dead: HashMap<Id, DeadPerson>,
    new_ids: Vec<Id>,
}

pub struct PopIterOrder {
    ids: Vec<Id>,
}

impl Population {
    /// Create a new population with pre-allocated buffers to accomodate the given population size.
    pub fn for_npersons(npersons: usize) -> (Population, PopIterOrder) {
        // This is a naive heuristic that might need be refined.
        let pop = Self {
            living: HashMap::with_capacity(npersons / 2 * 3),
            dead: HashMap::with_capacity(npersons / 2),
            new_ids: Vec::with_capacity(npersons),
        };
        let order = PopIterOrder {
            ids: Vec::with_capacity(npersons / 2 * 3),
        };
        (pop, order)
    }

    /// Number of living persons.
    pub fn size(&self) -> usize {
        self.living.len()
    }

    /// Add an individual to the population.
    pub fn insert(&mut self, person: Person) {
        let id = person.id();
        self.new_ids.push(id);
        self.living.insert(id, person);
    }

    pub fn mark_as_dead(&mut self, id: Id) {
        let person = self
            .living
            .remove(&id)
            .expect("{id:?} is not a living person");
        // FIXME: can this be left to grow or will need to be pruned as well?
        let dead = person.into();
        self.dead.insert(id, dead);
    }

    pub fn alive(&self, id: Id) -> &Person {
        self.living.get(&id).expect("{id:?} is not a living person")
    }

    pub fn alive_mut(&mut self, id: Id) -> &mut Person {
        self.living
            .get_mut(&id)
            .expect("{id:?} is not a living person")
    }

    pub fn alives<'a>(&'a self, order: &PopIterOrder) -> impl Iterator<Item = &'a Person> {
        order.ids().map(|id| self.alive(id))
    }

    pub fn for_each<F: FnMut(&mut Person)>(&mut self, order: &PopIterOrder, mut f: F) {
        order.ids().for_each(|id| {
            let person = self
                .living
                .get_mut(&id)
                .expect("{id:?} is not a living person");
            f(person)
        });
    }

    pub fn get(&self, id: Id) -> AliveOrDead<'_> {
        if let Some(p) = self.living.get(&id) {
            AliveOrDead::Alive(p)
        } else {
            let p = self
                .dead
                .get(&id)
                .expect("{id:?} is neither a living nor a dead person");
            AliveOrDead::Dead(p)
        }
    }
}

impl PopIterOrder {
    /// Register new individuals in iteration order and shuffle.
    pub fn register_new<R: Rng>(&mut self, pop: &mut Population, rng: &mut R) {
        if pop.new_ids.is_empty() {
            return;
        }
        self.ids.reserve(pop.new_ids.len());
        while let Some(id) = pop.new_ids.pop() {
            self.ids.push(id);
        }
        self.shuffle(rng);
    }

    // FIXME: make available only in create_pyramid_population for initial construction
    pub fn insert(&mut self, id: Id) {
        self.ids.push(id);
    }

    pub fn ids(&self) -> impl Iterator<Item = Id> {
        self.ids.iter().copied()
    }

    pub fn register_dead(&mut self, pop: &mut Population) {
        self.ids.retain(|id| pop.living.contains_key(id))
    }

    fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.ids.shuffle(rng);
    }
}
