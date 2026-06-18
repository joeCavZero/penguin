use crate::cell::PengCell;

#[derive(Debug, Clone)]
pub struct PengVector {
    values: Vec<PengCell>
}

impl PengVector {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: PengCell) {
        self.values.push(value);
    }
}