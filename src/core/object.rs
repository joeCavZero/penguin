use std::collections::HashMap;
use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub struct PengObject {
    values: HashMap<PengNamePoolPtr, PengCell>
}