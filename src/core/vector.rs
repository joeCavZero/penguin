use crate::utils::*;

#[derive(Debug, Clone, PartialEq)]
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
}
