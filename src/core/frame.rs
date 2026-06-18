use std::collections::HashMap;

use crate::utils::*;
use crate::cell::*;

#[derive(Debug, Clone)]
pub enum PengFrame {
    Bytecode(PengBytecodeFrame),
    Native(PengNativeFrame),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFrame {
    pub instruction_counter: usize,
    pub base: usize,
    pub function: PengValuePtr,
    
    pub generics: Vec<PengCell>,
    pub env_params: Vec<PengCell>,
    pub params: Vec<PengCell>,
    pub locals: Vec<PengCell>,

    pub value_table: HashMap<PengNamePoolPtr, PengFrameValue>
}

#[derive(Debug, Clone)]
pub struct PengNativeFrame {
    pub function: PengValuePtr,
}

#[derive(Debug, Clone)]
pub enum PengFrameValue {
    Generic(usize),
    EnvParam(usize),
    Param(usize),
}