use crate::cell::*;
use crate::frame::*;

#[derive(Debug, Clone)]
pub struct PengThread {
    pub stack: Vec<PengCell>,
    pub frames: Vec<PengFrame>,
}