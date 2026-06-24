use std::collections::HashMap;
use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengModule {
    pub members: HashMap<PengNamePoolPtr, PengBindedStatedCell>
}

impl PengModule {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.members.len() == rhs.members.len()
            && self.members.iter().all(|(name, value)| {
                match rhs.members.get(name) {
                    Some(rhs_value) => value.equals(rhs_value),
                    None => false,
                }
            })
    }
}