use std::collections::HashMap;
use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengObject {
    pub fields: HashMap<PengNamePoolPtr, PengBindedCell>
}

impl PengObject {
    pub fn new(fields: HashMap<PengNamePoolPtr, PengBindedCell>) -> Self {
        Self { fields }
    }
    pub fn new_empty() -> Self {
        Self {
            fields: HashMap::new()
        }
    }
    pub fn equals(&self, rhs: &Self) -> bool {
        self.fields.len() == rhs.fields.len()
            && self
                .fields
                .iter()
                .all(
                    |(name, value)| {
                        match rhs.fields.get(name) {
                            Some(rhs_value) => value.equals(rhs_value),
                            None => false,
                        } 
                    }
                )
    }
}
