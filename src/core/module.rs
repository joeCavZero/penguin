use std::collections::HashMap;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PengModule {
    values: HashMap<PengNamePoolPtr, PengValuePtr>
}
