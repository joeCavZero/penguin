use std::collections::HashMap;

use crate::core::utils::*;
use crate::core::cell::*;

pub struct PengUnit {
    pub init: PengHeapPtr,
    pub globals: HashMap<PengNamePoolPtr, PengBindedCell>,
}