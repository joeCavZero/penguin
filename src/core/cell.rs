use crate::utils::*;
use crate::binding::*;

pub type PengBindedCell = PengBinded<PengCell>;

#[derive(Debug, Clone)]
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

impl PengCell {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Nil, Self::Nil) => true,
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Uint(left), Self::Uint(right)) => left == right,
            (Self::Float32(left), Self::Float32(right)) => left == right,
            (Self::Float64(left), Self::Float64(right)) => left == right,
            (Self::Byte(left), Self::Byte(right)) => left == right,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Reference(left), Self::Reference(right)) => left == right,
            _ => false,
        }
    }
}
