use std::rc::Rc;

use crate::core::position::*;
use crate::core::cell::*;
use crate::core::error::*;
use crate::core::instruction::*;
use crate::core::utils::*;
use crate::core::value::*;
use crate::core::context::*;

#[derive(Debug, Clone)]
pub enum PengFunction {
    Bytecode(PengBytecodeFunction),
    Native(PengNativeFunction),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFunction {
    pub bytecode: Vec<PengInstruction>,
    pub positions: Vec<PengPosition>,
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
    pub fn new_native<F>(f: F) -> Self
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        Self::Native(PengNativeFunction {
            call: Rc::new(f),
        })
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

#[derive(Clone)]
pub struct PengNativeFunction {
    pub call: Rc<
        dyn Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError>,
    >,
}

impl PengNativeFunction {
    pub fn call(
        &self,
        ctx: &mut PengNativeFunctionCallContext,
    ) -> Result<PengBindedCell, PengError> {
        (self.call)(ctx)
    }
}

impl std::fmt::Debug for PengNativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PengNativeFunction").finish()
    }
}
