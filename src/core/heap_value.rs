use crate::colour::*;
use crate::function::*;
use crate::module::*;
use crate::object::*;
use crate::operation::*;
use crate::thread::*;
use crate::typing::*;
use crate::unioning::*;
use crate::vector::*;

pub type PengColouredHeapValue = PengColoured<PengHeapValue>;

#[derive(Debug, Clone)]
pub enum PengHeapValue {
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

impl PengHeapValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (PengHeapValue::String(a), PengHeapValue::String(b)) => a == b,

            (PengHeapValue::Object(a), PengHeapValue::Object(b)) => a.equals(b),
            (PengHeapValue::Vector(a), PengHeapValue::Vector(b)) => a.equals(b),
            (PengHeapValue::Type(a), PengHeapValue::Type(b)) => a.equals(b),
            (PengHeapValue::Module(a), PengHeapValue::Module(b)) => a.equals(b),
            (PengHeapValue::Thread(a), PengHeapValue::Thread(b)) => a.equals(b),
            (PengHeapValue::Function(a), PengHeapValue::Function(b)) => a.equals(b),
            (PengHeapValue::Operation(a), PengHeapValue::Operation(b)) => a.equals(b),
            (PengHeapValue::Union(a), PengHeapValue::Union(b)) => a.equals(b),

            _ => false,
        }
    }
}

impl PengColouredHeapValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.value.equals(&rhs.value)
    }
}
