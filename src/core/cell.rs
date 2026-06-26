use crate::core::binding::*;
use crate::core::error::*;
use crate::core::utils::*;

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
    Reference(PengHeapPtr),
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

    pub fn greater_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left > right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left > right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left > right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left > right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left > right),

            _ => Err(PengError::InvalidBinaryOperationCell {
                operator: ">".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn greater_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left >= right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left >= right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left >= right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left >= right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left >= right),

            _ => Err(PengError::InvalidBinaryOperationCell {
                operator: ">=".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn less_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left < right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left < right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left < right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left < right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left < right),

            _ => Err(PengError::InvalidBinaryOperationCell {
                operator: "<".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn less_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left <= right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left <= right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left <= right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left <= right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left <= right),

            _ => Err(PengError::InvalidBinaryOperationCell {
                operator: "<=".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }
}

impl PengBindedCell {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Mutable(left), Self::Mutable(right)) => left.equals(right),
            (Self::Immutable(left), Self::Immutable(right)) => left.equals(right),
            _ => false,
        }
    }
}
