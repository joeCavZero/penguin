use std::collections::HashMap;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct PengObject {
    values: HashMap<PengNamePoolPtr, PengValuePtr>
}