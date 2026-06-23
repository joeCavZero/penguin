use std::collections::HashMap;

use crate::core::*;

#[derive(Debug, Clone)]
pub enum PengType {
    Nil,

    Int,
    Uint,
    Float32,
    Float64,
    Byte,
    Bool,
    String,

    Object,
    Vector(Box<PengType>),

    Type,
    Module,

    Function,
    Operator,
    Thread,

    Custom(PengCustomType),

    Any,
    Union,
}

#[derive(Debug, Clone)]
pub struct PengCustomType {
    pub fields: HashMap<PengNamePoolPtr, PengBindedStatedCell>,
}

impl PengType {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Vector(left), Self::Vector(right)) => left.equals(right),
            (Self::Custom(left), Self::Custom(right)) => left.equals(right),
            (Self::Nil, Self::Nil)
            | (Self::Int, Self::Int)
            | (Self::Uint, Self::Uint)
            | (Self::Float32, Self::Float32)
            | (Self::Float64, Self::Float64)
            | (Self::Byte, Self::Byte)
            | (Self::Bool, Self::Bool)
            | (Self::String, Self::String)
            | (Self::Object, Self::Object)
            | (Self::Type, Self::Type)
            | (Self::Module, Self::Module)
            | (Self::Function, Self::Function)
            | (Self::Operator, Self::Operator)
            | (Self::Thread, Self::Thread)
            | (Self::Any, Self::Any) => true,
            _ => false,
        }
    }
}

impl PengCustomType {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.fields.len() == rhs.fields.len()
            && self.fields.iter().all(|(name, value)| {
                match rhs.fields.get(name) {
                    Some(rhs_value) => value.equals(rhs_value),
                    None => false,
                }
            })
    }
}
