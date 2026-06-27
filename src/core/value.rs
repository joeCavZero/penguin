use crate::core::*;

pub type PengColouredValue = PengColoured<PengValue>;

#[derive(Debug, Clone)]
pub enum PengValue {
    Cell(PengCell),
    Box(PengBox),
}

impl PengValue {
    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.equals(right),
            (Self::Box(left), Self::Box(right)) => left.equals(right),
            _ => false,
        }
    }

    pub fn greater_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.greater_than(right),
            _ => Err(PengError::InvalidBinaryOperationValue {
                operator: ">".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn greater_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.greater_equals_than(right),
            _ => Err(PengError::InvalidBinaryOperationValue {
                operator: ">=".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn less_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.less_than(right),
            _ => Err(PengError::InvalidBinaryOperationValue {
                operator: "<".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn less_equals_than(&self, rhs: &Self) -> Result<bool, PengError> {
        match (self, rhs) {
            (Self::Cell(left), Self::Cell(right)) => left.less_equals_than(right),
            _ => Err(PengError::InvalidBinaryOperationValue {
                operator: "<=".to_string(),
                left: self.clone(),
                right: rhs.clone(),
            }),
        }
    }

    pub fn convert(self, target_type: PengType) -> Result<PengValue, PengError> {
        let from = self.clone();
        let to = target_type.clone();

        match (self, target_type) {
            // Any
            (value, PengType::Any) => Ok(value),

            // Igual para igual
            (PengValue::Cell(PengCell::Nil), PengType::Nil) => Ok(PengValue::Cell(PengCell::Nil)),

            (PengValue::Cell(PengCell::Int(value)), PengType::Int) => {
                Ok(PengValue::Cell(PengCell::Int(value)))
            }

            (PengValue::Cell(PengCell::Uint(value)), PengType::Uint) => {
                Ok(PengValue::Cell(PengCell::Uint(value)))
            }

            (PengValue::Cell(PengCell::Byte(value)), PengType::Byte) => {
                Ok(PengValue::Cell(PengCell::Byte(value)))
            }

            (PengValue::Cell(PengCell::Float32(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value)))
            }

            (PengValue::Cell(PengCell::Float64(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value)))
            }

            (PengValue::Cell(PengCell::Bool(value)), PengType::Bool) => {
                Ok(PengValue::Cell(PengCell::Bool(value)))
            }

            // Int -> outros numéricos
            (PengValue::Cell(PengCell::Int(value)), PengType::Uint) => {
                match usize::try_from(value) {
                    Ok(value) => Ok(PengValue::Cell(PengCell::Uint(value))),
                    Err(_) => Err(PengError::InvalidConversion { from, to }),
                }
            }

            (PengValue::Cell(PengCell::Int(value)), PengType::Byte) => match u8::try_from(value) {
                Ok(value) => Ok(PengValue::Cell(PengCell::Byte(value))),
                Err(_) => Err(PengError::InvalidConversion { from, to }),
            },

            (PengValue::Cell(PengCell::Int(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value as f32)))
            }

            (PengValue::Cell(PengCell::Int(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            // Uint -> outros numéricos
            (PengValue::Cell(PengCell::Uint(value)), PengType::Int) => {
                match isize::try_from(value) {
                    Ok(value) => Ok(PengValue::Cell(PengCell::Int(value))),
                    Err(_) => Err(PengError::InvalidConversion { from, to }),
                }
            }

            (PengValue::Cell(PengCell::Uint(value)), PengType::Byte) => match u8::try_from(value) {
                Ok(value) => Ok(PengValue::Cell(PengCell::Byte(value))),
                Err(_) => Err(PengError::InvalidConversion { from, to }),
            },

            (PengValue::Cell(PengCell::Uint(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value as f32)))
            }

            (PengValue::Cell(PengCell::Uint(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            // Byte -> outros numéricos
            (PengValue::Cell(PengCell::Byte(value)), PengType::Int) => {
                Ok(PengValue::Cell(PengCell::Int(value as isize)))
            }

            (PengValue::Cell(PengCell::Byte(value)), PengType::Uint) => {
                Ok(PengValue::Cell(PengCell::Uint(value as usize)))
            }

            (PengValue::Cell(PengCell::Byte(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value as f32)))
            }

            (PengValue::Cell(PengCell::Byte(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            // Float32 -> outros numéricos
            (PengValue::Cell(PengCell::Float32(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            (PengValue::Cell(PengCell::Float32(value)), PengType::Int) => {
                if value.is_finite() && value >= isize::MIN as f32 && value <= isize::MAX as f32 {
                    Ok(PengValue::Cell(PengCell::Int(value as isize)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            (PengValue::Cell(PengCell::Float32(value)), PengType::Uint) => {
                if value.is_finite() && value >= 0.0 && value <= usize::MAX as f32 {
                    Ok(PengValue::Cell(PengCell::Uint(value as usize)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            (PengValue::Cell(PengCell::Float32(value)), PengType::Byte) => {
                if value.is_finite() && value >= u8::MIN as f32 && value <= u8::MAX as f32 {
                    Ok(PengValue::Cell(PengCell::Byte(value as u8)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            // Float64 -> outros numéricos
            (PengValue::Cell(PengCell::Float64(value)), PengType::Float32) => {
                if value.is_finite() && value >= f32::MIN as f64 && value <= f32::MAX as f64 {
                    Ok(PengValue::Cell(PengCell::Float32(value as f32)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            (PengValue::Cell(PengCell::Float64(value)), PengType::Int) => {
                if value.is_finite() && value >= isize::MIN as f64 && value <= isize::MAX as f64 {
                    Ok(PengValue::Cell(PengCell::Int(value as isize)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            (PengValue::Cell(PengCell::Float64(value)), PengType::Uint) => {
                if value.is_finite() && value >= 0.0 && value <= usize::MAX as f64 {
                    Ok(PengValue::Cell(PengCell::Uint(value as usize)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            (PengValue::Cell(PengCell::Float64(value)), PengType::Byte) => {
                if value.is_finite() && value >= u8::MIN as f64 && value <= u8::MAX as f64 {
                    Ok(PengValue::Cell(PengCell::Byte(value as u8)))
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            // Heap types
            (PengValue::Box(PengBox::String(value)), PengType::String) => {
                Ok(PengValue::Box(PengBox::String(value)))
            }

            (PengValue::Box(PengBox::Object(value)), PengType::Object) => {
                Ok(PengValue::Box(PengBox::Object(value)))
            }

            (PengValue::Box(PengBox::Vector(value)), PengType::Vector(_)) => {
                Ok(PengValue::Box(PengBox::Vector(value)))
            }

            (PengValue::Box(PengBox::Type(value)), PengType::Type) => {
                Ok(PengValue::Box(PengBox::Type(value)))
            }

            (PengValue::Box(PengBox::Module(value)), PengType::Module) => {
                Ok(PengValue::Box(PengBox::Module(value)))
            }

            (PengValue::Box(PengBox::Function(value)), PengType::Function) => {
                Ok(PengValue::Box(PengBox::Function(value)))
            }

            (PengValue::Box(PengBox::Operation(value)), PengType::Operator) => {
                Ok(PengValue::Box(PengBox::Operation(value)))
            }

            (PengValue::Box(PengBox::Thread(value)), PengType::Thread) => {
                Ok(PengValue::Box(PengBox::Thread(value)))
            }

            (PengValue::Box(PengBox::Union(value)), PengType::Union) => {
                Ok(PengValue::Box(PengBox::Union(value)))
            }

            // Qualquer tipo -> String
            (value, PengType::String) => {
                let value = match value {
                    PengValue::Cell(cell) => match cell {
                        PengCell::Nil => "nil".to_string(),
                        PengCell::Int(value) => value.to_string(),
                        PengCell::Uint(value) => value.to_string(),
                        PengCell::Byte(value) => value.to_string(),
                        PengCell::Float32(value) => value.to_string(),
                        PengCell::Float64(value) => value.to_string(),
                        PengCell::Bool(value) => value.to_string(),
                        PengCell::Reference(_) => {
                            return Err(PengError::InvalidConversion { from, to });
                        }
                    },

                    PengValue::Box(heap) => match heap {
                        PengBox::String(value) => value,

                        PengBox::Object(_)
                        | PengBox::Vector(_)
                        | PengBox::Type(_)
                        | PengBox::Module(_)
                        | PengBox::Thread(_)
                        | PengBox::Function(_)
                        | PengBox::Operation(_)
                        | PengBox::Union(_) => {
                            return Err(PengError::InvalidConversion { from, to });
                        }
                    },
                };

                Ok(PengValue::Box(PengBox::String(value)))
            }

            _ => Err(PengError::InvalidConversion { from, to }),
        }
    }
}
