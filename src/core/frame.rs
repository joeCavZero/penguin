
use crate::utils::*;
use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengFrame {
    pub program_counter: usize,
    pub base: usize,
    pub function: PengHeapPtr,
    pub params_count: usize,
}

impl PengFrame {
    pub fn new(function_ptr: PengHeapPtr, base: usize, params_count: usize) -> Self {
        PengFrame {
            program_counter: 0,
            base,
            function: function_ptr,
            params_count,
        }
    }
    pub fn equals(&self, rhs: &Self) -> bool {
        self.program_counter == rhs.program_counter
            && self.base == rhs.base
            && self.function == rhs.function
            && self.params_count == rhs.params_count
    }
}
