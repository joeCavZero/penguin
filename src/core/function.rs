use crate::utils::*;
use crate::instruction::*;
use crate::context::*;
use crate::cell::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub enum PengFunction {
    Bytecode(PengBytecodeFunction),
    Native(
        fn(&mut PengNativeFunctionContext) -> Result<Option<PengCell>, PengError>
    ),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub env_param_names: Vec<PengNamePoolPtr>,
    pub using_values: Vec<PengValuePtr>,
}