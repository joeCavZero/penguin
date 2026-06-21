use crate::cell::*;
use crate::frame::*;

#[derive(Debug, Clone)]
pub struct PengThread {
    pub stack: Vec<PengCell>,
    pub frames: Vec<PengFrame>,
}

impl PengThread {
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
