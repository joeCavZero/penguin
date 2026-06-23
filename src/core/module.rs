use std::collections::HashMap;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct PengModule {
    values: HashMap<PengNamePoolPtr, PengHeapPtr>
}

impl PengModule {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.values.len() == rhs.values.len()
            && self
                .values
                .iter()
                .all(|(name, value)| rhs.values.get(name) == Some(value))
    }
}
