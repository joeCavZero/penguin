use crate::core::utils::*;
use crate::core::cell::*;
use crate::core::frame::*;

#[derive(Debug, Clone)]
pub struct PengThread {
    pub stack: Vec<PengBindedCell>,
    pub frames: Vec<PengFrame>,
}

impl PengThread {
    pub fn new(function_ptr: PengHeapPtr, base: usize, params: Vec<PengBindedCell>) -> Self {
        
        let mut frames = Vec::new();

        frames.push(
            PengFrame::new(function_ptr, base, params.len())
        );
        
        let mut thread = PengThread {
            stack: Vec::new(),
            frames,
        };

        for p in &params {
            thread.stack.push(p.clone());
        }
        
        
        
        thread
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.stack.len() == rhs.stack.len()
            && self
                .stack
                .iter()
                .zip(&rhs.stack)
                .all(|(left, right)| left.equals(right))
            && self.frames.len() == rhs.frames.len()
            && self
                .frames
                .iter()
                .zip(&rhs.frames)
                .all(|(left, right)| left.equals(right))
    }
}
