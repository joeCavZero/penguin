use std::collections::HashMap;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct PengModule {
    values: HashMap<PengNamePoolPtr, PengValuePtr>
}