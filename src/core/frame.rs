
use crate::core::*;

#[derive(Debug, Clone)]
pub enum PengFrame {
    Bytecode(PengBytecodeFrame),
    Native(PengNativeFrame),
}

#[derive(Debug, Clone)]
pub struct PengBytecodeFrame {
    pub program_counter: usize,
    pub base: usize,
    pub function: PengHeapPtr,
    pub params_count: usize,
    pub error: Option<PengError>,
}

#[derive(Debug, Clone)]
pub struct PengNativeFrame {
    pub base: usize,
    pub function: PengHeapPtr,
    pub params_count: usize,
    pub error: Option<PengError>,
}

impl PengFrame {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bytecode(left), Self::Bytecode(right)) => left.equals(right),
            (Self::Native(left), Self::Native(right)) => left.equals(right),
            _ => false,
        }
    }
}

impl PengBytecodeFrame {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.program_counter == rhs.program_counter
            && self.base == rhs.base
            && self.function == rhs.function
            && self.params_count == rhs.params_count
            && errors_equal(&self.error, &rhs.error)
    }
}

impl PengNativeFrame {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.base == rhs.base
            && self.function == rhs.function
            && self.params_count == rhs.params_count
            && errors_equal(&self.error, &rhs.error)
    }
}

fn errors_equal(left: &Option<PengError>, right: &Option<PengError>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.equals(right),
        (None, None) => true,
        _ => false,
    }
}
