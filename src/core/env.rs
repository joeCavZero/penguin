use std::collections::HashMap;

use crate::utils::*;
use crate::value::*;

pub struct PengEnv {
    values: HashMap<PengValuePtr, PengValue>,
    name_pool: HashMap<PengNamePoolPtr, String>,
}