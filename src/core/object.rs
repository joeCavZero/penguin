use std::collections::HashMap;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PengObject {
    values: HashMap<PengNamePoolPtr, PengValuePtr>
}
