use crate::utils::*;

#[derive(Debug, Clone)]
pub struct PengVector {
    values: Vec<PengValuePtr>
}

impl PengVector {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: PengValuePtr) {
        self.values.push(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<PengValuePtr> {
        self.values.get(index).copied()
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.values.len() == rhs.values.len()
            && self
                .values
                .iter()
                .zip(&rhs.values)
                .all(|(left, right)| left == right)
    }
}
