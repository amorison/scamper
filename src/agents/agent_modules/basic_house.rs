use crate::{agents::towns::IdTown, full_model::person::Id};

#[derive(Clone, Copy)]
pub struct Location(usize, usize);

impl Location {
    pub fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }
}

pub struct BasicHouse {
    town: IdTown,
    pos: Location,
    occupants: Vec<Id>,
}

impl BasicHouse {
    pub fn new(town: IdTown, pos: Location) -> Self {
        Self {
            town,
            pos,
            occupants: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.occupants.is_empty()
    }

    pub fn is_occupied(&self) -> bool {
        !self.is_empty()
    }

    pub fn town(&self) -> IdTown {
        self.town
    }

    pub fn location(&self) -> Location {
        self.pos
    }

    pub fn occupants(&self) -> &[Id] {
        &self.occupants
    }

    pub fn add_occupant(&mut self, id: Id) {
        self.occupants.push(id)
    }

    pub fn rm_occupant(&mut self, id: Id) {
        let idx = self
            .occupants
            .iter()
            .position(|&i| id == i)
            .expect("id should be an occupant");
        self.occupants.swap_remove(idx);
    }
}
