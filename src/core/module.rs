use std::collections::HashMap;
use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengModule {
    values: HashMap<PengNamePoolPtr, PengCell>
}