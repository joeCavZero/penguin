use std::rc::Rc;

use crate::core::PengNativeOperationCallContext;
use crate::core::cell::*;
use crate::core::error::*;
use crate::core::instruction::*;
use crate::core::position::*;
use crate::core::utils::*;
use crate::core::value::*;

#[derive(Debug, Clone)]
pub enum PengOperation {
    Bytecode(PengBytecodeOperation),
    Native(PengNativeOperation),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeOperation {
    pub bytecode: Vec<PengInstruction>,
    pub positions: Vec<PengPosition>,
    pub consts: Vec<PengValue>,
    pub using_values: Vec<PengHeapPtr>,
}

impl PengOperation {
    pub fn new_native<F>(f: F) -> Self
    where
        F: Fn(&mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        Self::Native(PengNativeOperation { call: Rc::new(f) })
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bytecode(left), Self::Bytecode(right)) => left.equals(right),
            (Self::Native(_left), Self::Native(_right)) => false,
            _ => false,
        }
    }
}

impl PengBytecodeOperation {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.bytecode.len() == rhs.bytecode.len()
            && self
                .bytecode
                .iter()
                .zip(&rhs.bytecode)
                .all(|(left, right)| left.equals(right))
            && self.consts.len() == rhs.consts.len()
            && self
                .consts
                .iter()
                .zip(&rhs.consts)
                .all(|(left, right)| left.equals(right))
            && self.using_values.len() == rhs.using_values.len()
            && self
                .using_values
                .iter()
                .zip(&rhs.using_values)
                .all(|(left, right)| left == right)
    }
}

#[derive(Clone)]
pub struct PengNativeOperation {
    pub call: Rc<dyn Fn(&mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError>>,
}

impl PengNativeOperation {
    pub fn call(
        &self,
        ctx: &mut PengNativeOperationCallContext,
    ) -> Result<PengBindedCell, PengError> {
        (self.call)(ctx)
    }
}

impl std::fmt::Debug for PengNativeOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PengNativeOperation").finish()
    }
}
