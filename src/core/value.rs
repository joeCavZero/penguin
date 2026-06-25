use crate::core::*;

#[derive(Debug, Clone)]
pub enum PengValue {
    Cell(PengCell),
    Heap(PengHeapValue),
}

impl PengValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.equals(right),
            (Self::Heap(left), Self::Heap(right)) => left.equals(right),
            _ => false,
        }
    }

    pub fn greater_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.greater_than(right),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn greater_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.greater_equals_than(right),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn less_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.less_than(right),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn less_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.less_equals_than(right),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn convert_value(self, target_type: PengType) -> Result<PengValue, PengError> {
        match (self, target_type) {
            (PengValue::Cell(PengCell::Nil), PengType::Nil) => Ok(PengValue::Cell(PengCell::Nil)),
            (PengValue::Cell(PengCell::Int(value)), PengType::Int) => {
                Ok(PengValue::Cell(PengCell::Int(value)))
            }
            (PengValue::Cell(PengCell::Uint(value)), PengType::Uint) => {
                Ok(PengValue::Cell(PengCell::Uint(value)))
            }
            (PengValue::Cell(PengCell::Float32(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value)))
            }
            (PengValue::Cell(PengCell::Float64(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value)))
            }
            (PengValue::Cell(PengCell::Byte(value)), PengType::Byte) => {
                Ok(PengValue::Cell(PengCell::Byte(value)))
            }
            (PengValue::Cell(PengCell::Bool(value)), PengType::Bool) => {
                Ok(PengValue::Cell(PengCell::Bool(value)))
            }
            (PengValue::Heap(PengHeapValue::String(value)), PengType::String) => {
                Ok(PengValue::Heap(PengHeapValue::String(value)))
            }
            (PengValue::Heap(PengHeapValue::Object(value)), PengType::Object) => {
                Ok(PengValue::Heap(PengHeapValue::Object(value)))
            }
            (PengValue::Heap(PengHeapValue::Vector(value)), PengType::Vector(_)) => {
                Ok(PengValue::Heap(PengHeapValue::Vector(value)))
            }
            (PengValue::Heap(PengHeapValue::Type(value)), PengType::Type) => {
                Ok(PengValue::Heap(PengHeapValue::Type(value)))
            }
            (PengValue::Heap(PengHeapValue::Module(value)), PengType::Module) => {
                Ok(PengValue::Heap(PengHeapValue::Module(value)))
            }
            (PengValue::Heap(PengHeapValue::Function(value)), PengType::Function) => {
                Ok(PengValue::Heap(PengHeapValue::Function(value)))
            }
            (PengValue::Heap(PengHeapValue::Operation(value)), PengType::Operator) => {
                Ok(PengValue::Heap(PengHeapValue::Operation(value)))
            }
            (PengValue::Heap(PengHeapValue::Thread(value)), PengType::Thread) => {
                Ok(PengValue::Heap(PengHeapValue::Thread(value)))
            }
            (PengValue::Heap(PengHeapValue::Union(value)), PengType::Union) => {
                Ok(PengValue::Heap(PengHeapValue::Union(value)))
            }
            (value, PengType::Any) => Ok(value),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }
}
