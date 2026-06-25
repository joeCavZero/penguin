use crate::value::*;
use crate::utils::*;
use crate::instruction::*;
use crate::context::*;
use crate::cell::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub enum PengFunction {
    Bytecode(PengBytecodeFunction),
    Native(
        fn(&mut PengNativeCallContext) -> Result<PengCell, PengError>
    ),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengHeapValue>,

    pub using_values: Vec<PengHeapPtr>,
}

impl PengFunction {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bytecode(left), Self::Bytecode(right)) => left.equals(right),
            (Self::Native(left), Self::Native(right)) => {
                std::ptr::fn_addr_eq(*left, *right)
            }
            _ => false,
        }
    }
}

impl PengBytecodeFunction {
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
