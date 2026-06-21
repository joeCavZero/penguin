use std::collections::HashMap;

use crate::utils::*;

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
}

#[derive(Debug, Clone)]
pub struct PengCustomType {
    pub fields: HashMap<PengNamePoolPtr, Option<PengValuePtr>>,
}