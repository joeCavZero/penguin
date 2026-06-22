use crate::vector::*;
use crate::object::*;
use crate::module::*;
use crate::typing::*;
use crate::thread::*;
use crate::function::*;
use crate::operation::*;
use crate::unioning::*;
use crate::binding::*;

pub type PengBindedValue = PengBinded<PengValue>;

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
}

impl PengBinded<PengValue> {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (
                Self::Immutable(v1) | Self::Mutable(v1),
                Self::Immutable(v2) | Self::Mutable(v2),
            ) => v1.equals(v2),

            _ => false,
        }
    }
}