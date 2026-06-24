
use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengFrame {
    pub program_counter: usize,
    pub base: usize,
    pub procedure: PengHeapPtr,
    pub params_count: usize,
    pub is_try: bool,
}

impl PengFrame {
    pub fn new(procedure: PengHeapPtr, base: usize, params_count: usize) -> Self {
        PengFrame {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: false
        }
    }
    pub fn new_try(procedure: PengHeapPtr, base: usize, params_count: usize) -> Self {
        PengFrame {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: true
        }
    }
    pub fn equals(&self, rhs: &Self) -> bool {
        self.program_counter == rhs.program_counter
            && self.base == rhs.base
            && self.procedure == rhs.procedure
            && self.params_count == rhs.params_count
    }
}
