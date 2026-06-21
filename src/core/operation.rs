use crate::value::*;
use crate::utils::*;
use crate::instruction::*;
use crate::context::*;
use crate::cell::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub enum PengOperation {
    Bytecode(PengBytecodeOperation),
    Native(
        fn(&mut PengNativeCallContext) -> Result<PengCell, PengError>
    ),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PengBytecodeOperation {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub generics_count: usize,
    pub using_values: Vec<PengValuePtr>,
}

impl PartialEq for PengOperation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PengOperation::Bytecode(left),
                PengOperation::Bytecode(right),
            ) => left == right,
            (
                PengOperation::Native(left),
                PengOperation::Native(right),
            ) => std::ptr::fn_addr_eq(*left, *right),
            _ => false,
        }
    }
}
