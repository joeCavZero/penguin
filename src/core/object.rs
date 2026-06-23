use std::collections::HashMap;
use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengObject {
    values: HashMap<PengNamePoolPtr, PengBindedStatedCell>
}

impl PengObject {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.values.len() == rhs.values.len()
            && self
                .values
                .iter()
                .all(
                    |(name, value)| {
                        match rhs.values.get(name) {
                            Some(rhs_value) => value.equals(rhs_value),
                            None => false,
                        } 
                    }
                )
    }
}
