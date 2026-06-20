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
}