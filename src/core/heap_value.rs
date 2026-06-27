use crate::core::colour::*;
use crate::core::function::*;
use crate::core::module::*;
use crate::core::object::*;
use crate::core::operation::*;
use crate::core::thread::*;
use crate::core::typing::*;
use crate::core::unioning::*;
use crate::core::vector::*;

pub type PengColouredHeapValue = PengColoured<PengBox>;

#[derive(Debug, Clone)]
pub enum PengBox {
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

impl PengBox {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (PengBox::String(a), PengBox::String(b)) => a == b,

            (PengBox::Object(a), PengBox::Object(b)) => a.equals(b),
            (PengBox::Vector(a), PengBox::Vector(b)) => a.equals(b),
            (PengBox::Type(a), PengBox::Type(b)) => a.equals(b),
            (PengBox::Module(a), PengBox::Module(b)) => a.equals(b),
            (PengBox::Thread(a), PengBox::Thread(b)) => a.equals(b),
            (PengBox::Function(a), PengBox::Function(b)) => a.equals(b),
            (PengBox::Operation(a), PengBox::Operation(b)) => a.equals(b),
            (PengBox::Union(a), PengBox::Union(b)) => a.equals(b),

            _ => false,
        }
    }
}

impl PengColouredHeapValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        self.value.equals(&rhs.value)
    }
}
