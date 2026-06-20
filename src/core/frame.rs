
use crate::utils::*;

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
    pub params_count: usize,
}

#[derive(Debug, Clone)]
pub struct PengNativeFrame {
    pub base: usize,
    pub function: PengValuePtr,
    pub params_count: usize,
}