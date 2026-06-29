use std::time::Instant;

use crate::core::cell::*;
use crate::core::error::*;
use crate::core::frame::*;
use crate::core::utils::*;

#[derive(Debug, Clone)]
pub enum PengThreadState {
    Running,
    Finished,
    Paused,
    Waiting(PengHeapPtr),
    Cancelled,
    Failed,
    Sleeping(Instant),
}

#[derive(Debug, Clone)]
pub enum PengThreadResult {
    Pending,
    Returned(PengBindedCell),
    Failed(Box<PengError>),
}

#[derive(Debug, Clone)]
pub struct PengThread {
    pub state: PengThreadState,
    pub stack: Vec<PengBindedCell>,
    pub frames: Vec<PengFrame>,
    pub result: PengThreadResult,
    pub quantum: usize,
}

impl PengThread {
    pub fn new(
        function: PengHeapPtr,
        base: usize,
        params: Vec<PengBindedCell>,
        state: PengThreadState,
    ) -> Self {
        let mut frames = Vec::new();

        frames.push(PengFrame::new(function, base, params.len()));

        let mut thread = PengThread {
            state,
            stack: Vec::new(),
            frames,
            result: PengThreadResult::Pending,
            quantum: 1,
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
