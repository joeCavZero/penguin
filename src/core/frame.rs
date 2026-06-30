use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengFrame {
    pub program_counter: usize,
    pub base: usize,
    pub procedure: PengHeapPtr,
    pub params_count: usize,
    pub is_try: bool,
    pub reserved_locals: Vec<usize>,
    pub call_position: Option<PengPosition>,
}

impl PengFrame {
    pub fn new(procedure: PengHeapPtr, base: usize, params_count: usize) -> Self {
        PengFrame {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: false,
            reserved_locals: Vec::new(),
            call_position: None,
        }
    }
    pub fn new_positioned(
        procedure: PengHeapPtr,
        base: usize,
        params_count: usize,
        position: PengPosition,
    ) -> Self {
        PengFrame {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: false,
            reserved_locals: Vec::new(),
            call_position: Some(position),
        }
    }
    pub fn new_try(procedure: PengHeapPtr, base: usize, params_count: usize) -> Self {
        Self {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: true,
            reserved_locals: Vec::new(),
            call_position: None,
        }
    }

    pub fn new_try_positioned(
        procedure: PengHeapPtr,
        base: usize,
        params_count: usize,
        position: PengPosition,
    ) -> Self {
        Self {
            program_counter: 0,
            base,
            procedure,
            params_count,
            is_try: true,
            reserved_locals: Vec::new(),
            call_position: Some(position),
        }
    }

    pub fn new_positioned_optional(
        procedure: PengHeapPtr,
        base: usize,
        params_count: usize,
        call_position: Option<PengPosition>,
    ) -> Self {
        Self {
            procedure,
            base,
            params_count,
            program_counter: 0,
            is_try: false,
            reserved_locals: Vec::new(),
            call_position,
        }
    }

    pub fn new_try_positioned_optional(
        procedure: PengHeapPtr,
        base: usize,
        params_count: usize,
        position: Option<PengPosition>,
    ) -> Self {
        Self {
            procedure,
            base,
            params_count,
            program_counter: 0,
            is_try: true,
            reserved_locals: Vec::new(),
            call_position: position,
        }
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.program_counter == rhs.program_counter
            && self.base == rhs.base
            && self.procedure == rhs.procedure
            && self.params_count == rhs.params_count
            && self.reserved_locals == rhs.reserved_locals
    }
}
