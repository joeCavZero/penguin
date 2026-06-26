use crate::core::value::*;
use crate::core::utils::*;
use crate::core::instruction::*;
use crate::core::env::*;
use crate::core::cell::*;
use crate::core::error::*;

#[derive(Debug, Clone)]
pub enum PengOperation {
    Bytecode(PengBytecodeOperation),
    Native(PengNativeOperation),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeOperation {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub using_values: Vec<PengHeapPtr>,
}

impl PengOperation {
    pub fn new_native(f: fn((PengBindedCell, PengBindedCell),&mut PengEnv) -> Result<PengBindedCell, PengError>) -> Self{
        Self::Native(
            PengNativeOperation {
                call: f
            }
        )
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


#[derive(Debug, Clone)]
pub struct PengNativeOperation {
    pub call: fn(
        (PengBindedCell, PengBindedCell),
        &mut PengEnv,
    ) -> Result<PengBindedCell, PengError>,
}

impl PengNativeOperation {
    pub fn call(
        &self,
        args: (PengBindedCell, PengBindedCell),
        env: &mut PengEnv,
    ) -> Result<PengBindedCell, PengError> {
        (self.call)(args, env)
    }
}