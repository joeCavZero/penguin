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

#[derive(Debug, Clone)]
pub struct PengBytecodeOperation {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub generics_count: usize,
    pub using_values: Vec<PengValuePtr>,
}