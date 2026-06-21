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

#[derive(Debug, Clone, PartialEq)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub generics_count: usize,
    pub using_values: Vec<PengValuePtr>,
}

impl PartialEq for PengFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PengFunction::Bytecode(left),
                PengFunction::Bytecode(right),
            ) => left == right,
            (
                PengFunction::Native(left),
                PengFunction::Native(right),
            ) => std::ptr::fn_addr_eq(*left, *right),
            _ => false,
        }
    }
}
