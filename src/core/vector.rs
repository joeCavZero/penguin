use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengVector {
    pub values: Vec<PengBindedCell>
}

impl PengVector {
    pub fn new(values: Vec<PengBindedCell>) -> Self {
        Self { values }
    }
    pub fn new_empty() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: PengBindedCell) {
        self.values.push(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&PengBindedCell> {
        self.values.get(index)
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.values.len() == rhs.values.len()
            && self
                .values
                .iter()
                .zip(&rhs.values)
                .all(|(left, right)| left.equals(right))
    }
}
