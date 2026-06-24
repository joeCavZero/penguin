use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengVector {
    pub values: Vec<PengBindedStatedCell>
}

impl PengVector {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: PengBindedStatedCell) {
        self.values.push(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&PengBindedStatedCell> {
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
