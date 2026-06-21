use crate::cell::*;
use crate::frame::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PengThread {
    pub stack: Vec<PengCell>,
    pub frames: Vec<PengFrame>,
}
