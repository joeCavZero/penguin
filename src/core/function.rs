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
        fn(&mut PengNativeFunctionContext) -> Result<PengCell, PengError>
    ),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub generics_count: usize,
    pub using_values: Vec<PengValuePtr>,
}