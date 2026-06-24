use crate::binding::*;
use crate::cell::*;
use crate::colour::*;
use crate::error::*;
use crate::function::*;
use crate::module::*;
use crate::object::*;
use crate::operation::*;
use crate::state::*;
use crate::thread::*;
use crate::typing::*;
use crate::unioning::*;
use crate::vector::*;

pub type PengStatedValue = PengStated<PengValue>;

pub type PengBindedStatedValue = PengBinded<PengStatedValue>;

pub type PengColouredBindedStatedValue = PengColoured<PengBindedStatedValue>;

#[derive(Debug, Clone)]
pub enum PengValue {
    Nil,
    Int(isize),
    Uint(usize),
    Float32(f32),
    Float64(f64),
    Byte(u8),
    Bool(bool),
    String(String),
    Object(PengObject),
    Vector(PengVector),
    Type(PengType),
    Module(PengModule),
    Thread(PengThread),
    Function(PengFunction),
    Operation(PengOperation),
    Union(PengUnion),
}

impl PengValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (PengValue::Nil, PengValue::Nil) => true,

            (PengValue::Int(a), PengValue::Int(b)) => a == b,
            (PengValue::Uint(a), PengValue::Uint(b)) => a == b,
            (PengValue::Float32(a), PengValue::Float32(b)) => a == b,
            (PengValue::Float64(a), PengValue::Float64(b)) => a == b,
            (PengValue::Byte(a), PengValue::Byte(b)) => a == b,
            (PengValue::Bool(a), PengValue::Bool(b)) => a == b,
            (PengValue::String(a), PengValue::String(b)) => a == b,

            (PengValue::Object(a), PengValue::Object(b)) => a.equals(b),
            (PengValue::Vector(a), PengValue::Vector(b)) => a.equals(b),
            (PengValue::Type(a), PengValue::Type(b)) => a.equals(b),
            (PengValue::Module(a), PengValue::Module(b)) => a.equals(b),
            (PengValue::Thread(a), PengValue::Thread(b)) => a.equals(b),
            (PengValue::Function(a), PengValue::Function(b)) => a.equals(b),
            (PengValue::Operation(a), PengValue::Operation(b)) => a.equals(b),
            (PengValue::Union(a), PengValue::Union(b)) => a.equals(b),

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

            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn greater_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left >= right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left >= right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left >= right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left >= right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left >= right),

            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn less_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left < right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left < right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left < right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left < right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left < right),

            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn less_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Int(left), Self::Int(right)) => Ok(left <= right),
            (Self::Uint(left), Self::Uint(right)) => Ok(left <= right),
            (Self::Float32(left), Self::Float32(right)) => Ok(left <= right),
            (Self::Float64(left), Self::Float64(right)) => Ok(left <= right),
            (Self::Byte(left), Self::Byte(right)) => Ok(left <= right),

            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn is_heap(&self) -> bool {
        match self {
            Self::Object(_) => true,
            Self::Vector(_) => true,
            Self::Type(_) => true,
            Self::Module(_) => true,
            Self::Thread(_) => true,
            Self::Function(_) => true,
            Self::Operation(_) => true,
            Self::Union(_) => true,

            _ => false,
        }
    }

    pub fn to_cell(&self) -> Result<PengCell, PengError> {
        match self {
            Self::Nil => Ok(PengCell::Nil),
            Self::Int(v) => Ok(PengCell::Int(*v)),
            Self::Uint(v) => Ok(PengCell::Uint(*v)),
            Self::Float32(v) => Ok(PengCell::Float32(*v)),
            Self::Float64(v) => Ok(PengCell::Float64(*v)),
            Self::Byte(v) => Ok(PengCell::Byte(*v)),
            Self::Bool(v) => Ok(PengCell::Bool(*v)),

            _ => Err(PengError::new_message(
                "value cannot be converted to cell".to_string(),
            )),
        }
    }

    pub fn convert(self, target_type: PengType) -> Result<PengValue, PengError> {
        match target_type {
            PengType::Nil => Ok(PengValue::Nil),

            PengType::Int => match self {
                PengValue::Int(v) => Ok(PengValue::Int(v)),
                PengValue::Uint(v) => Ok(PengValue::Int(v as isize)),
                PengValue::Float32(v) => Ok(PengValue::Int(v as isize)),
                PengValue::Float64(v) => Ok(PengValue::Int(v as isize)),
                PengValue::Byte(v) => Ok(PengValue::Int(v as isize)),
                PengValue::Bool(v) => Ok(PengValue::Int(if v { 1 } else { 0 })),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Uint => match self {
                PengValue::Int(v) => Ok(PengValue::Uint(v as usize)),
                PengValue::Uint(v) => Ok(PengValue::Uint(v)),
                PengValue::Float32(v) => Ok(PengValue::Uint(v as usize)),
                PengValue::Float64(v) => Ok(PengValue::Uint(v as usize)),
                PengValue::Byte(v) => Ok(PengValue::Uint(v as usize)),
                PengValue::Bool(v) => Ok(PengValue::Uint(if v { 1 } else { 0 })),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Float32 => match self {
                PengValue::Int(v) => Ok(PengValue::Float32(v as f32)),
                PengValue::Uint(v) => Ok(PengValue::Float32(v as f32)),
                PengValue::Float32(v) => Ok(PengValue::Float32(v)),
                PengValue::Float64(v) => Ok(PengValue::Float32(v as f32)),
                PengValue::Byte(v) => Ok(PengValue::Float32(v as f32)),
                PengValue::Bool(v) => Ok(PengValue::Float32(if v { 1.0 } else { 0.0 })),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Float64 => match self {
                PengValue::Int(v) => Ok(PengValue::Float64(v as f64)),
                PengValue::Uint(v) => Ok(PengValue::Float64(v as f64)),
                PengValue::Float32(v) => Ok(PengValue::Float64(v as f64)),
                PengValue::Float64(v) => Ok(PengValue::Float64(v)),
                PengValue::Byte(v) => Ok(PengValue::Float64(v as f64)),
                PengValue::Bool(v) => Ok(PengValue::Float64(if v { 1.0 } else { 0.0 })),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Byte => match self {
                PengValue::Int(v) => Ok(PengValue::Byte(v as u8)),
                PengValue::Uint(v) => Ok(PengValue::Byte(v as u8)),
                PengValue::Float32(v) => Ok(PengValue::Byte(v as u8)),
                PengValue::Float64(v) => Ok(PengValue::Byte(v as u8)),
                PengValue::Byte(v) => Ok(PengValue::Byte(v)),
                PengValue::Bool(v) => Ok(PengValue::Byte(if v { 1 } else { 0 })),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Bool => match self {
                PengValue::Int(v) => Ok(PengValue::Bool(v != 0)),
                PengValue::Uint(v) => Ok(PengValue::Bool(v != 0)),
                PengValue::Float32(v) => Ok(PengValue::Bool(v != 0.0)),
                PengValue::Float64(v) => Ok(PengValue::Bool(v != 0.0)),
                PengValue::Byte(v) => Ok(PengValue::Bool(v != 0)),
                PengValue::Bool(v) => Ok(PengValue::Bool(v)),
                PengValue::Nil => Ok(PengValue::Bool(false)),
                _ => Ok(PengValue::Bool(true)),
            },

            PengType::String => match self {
                PengValue::String(v) => Ok(PengValue::String(v)),
                PengValue::Nil => Ok(PengValue::String("nil".to_string())),
                PengValue::Int(v) => Ok(PengValue::String(v.to_string())),
                PengValue::Uint(v) => Ok(PengValue::String(v.to_string())),
                PengValue::Float32(v) => Ok(PengValue::String(v.to_string())),
                PengValue::Float64(v) => Ok(PengValue::String(v.to_string())),
                PengValue::Byte(v) => Ok(PengValue::String(v.to_string())),
                PengValue::Bool(v) => Ok(PengValue::String(v.to_string())),
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            PengType::Any => Ok(self),

            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }
}

impl PengStatedValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Initialized(v1), Self::Initialized(v2)) => v1.equals(v2),
            _ => false,
        }
    }
}

impl PengBindedStatedValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Immutable(v1) | Self::Mutable(v1), Self::Immutable(v2) | Self::Mutable(v2)) => {
                v1.equals(v2)
            }
        }
    }
}

impl PengColouredBindedStatedValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.value.equals(&rhs.value)
    }
}
