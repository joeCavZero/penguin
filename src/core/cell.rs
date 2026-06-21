use crate::utils::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PengCell {
    Nil,
    Int(isize),
    Uint(usize),
    Float32(f32),
    Float64(f64),
    Byte(u8),
    Bool(bool),
    Reference(PengValuePtr),
}
