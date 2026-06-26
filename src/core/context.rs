use crate::core::env::*;
use crate::core::cell::*;
use crate::core::error::*;

pub struct PengNativeCallContext {}

impl PengNativeCallContext {
    pub fn equals(&self, _rhs: &Self) -> bool {
        true
    }
}

pub type PengNativeFn = fn(Vec<PengBindedCell>,&mut PengEnv) -> Result<PengBindedCell, PengError>;