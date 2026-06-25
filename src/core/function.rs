use crate::value::*;
use crate::utils::*;
use crate::instruction::*;
use crate::env::*;
use crate::cell::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub enum PengFunction {
    Bytecode(PengBytecodeFunction),
    Native(PengNativeFunction),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    pub using_values: Vec<PengHeapPtr>,

    pub params: PengBytecodeFunctionParams,
}

#[derive(Debug, Clone)]
pub enum PengBytecodeFunctionParams {
    Fixed(usize),
    Variadic(usize),
}

impl PengBytecodeFunctionParams {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Fixed(left), Self::Fixed(right)) => left == right,
            (Self::Variadic(left), Self::Variadic(right)) => left == right,
            _ => false,
        }
    }
}

impl PengFunction {
    pub fn new_native(f: fn(Vec<PengBindedCell>,&mut PengEnv) -> Result<PengBindedCell, PengError>) -> Self{
        Self::Native(
            PengNativeFunction {
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
            && self.params.equals(&rhs.params)
    }
}

#[derive(Debug, Clone)]
pub struct PengNativeFunction {
    pub call: fn(
        Vec<PengBindedCell>,
        &mut PengEnv,
    ) -> Result<PengBindedCell, PengError>,
}

impl PengNativeFunction {
    pub fn call(
        &self,
        args: Vec<PengBindedCell>,
        env: &mut PengEnv,
    ) -> Result<PengBindedCell, PengError> {
        (self.call)(args, env)
    }
}