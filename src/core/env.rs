use std::collections::HashMap;

use crate::utils::*;
use crate::value::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengValuePtr>,
    values: HashMap<PengValuePtr, PengValue>,
    name_pool: HashMap<PengNamePoolPtr, String>,
}