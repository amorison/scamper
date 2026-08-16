use crate::utilities::Age;

#[derive(Clone, Copy, Debug)]
pub enum Gender {
    Female,
    Male,
}

pub struct BasicInfo {
    pub age: Age,
    pub gender: Gender,
}

impl BasicInfo {
    pub fn new(gender: Gender, age: Age) -> Self {
        Self { age, gender }
    }

    pub fn has_birthday(&self) -> bool {
        let (_, month) = self.age.year_month();
        month.index() == 0
    }
}
