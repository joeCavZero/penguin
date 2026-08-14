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

            (Self::Box(PengBox::String(left)), Self::Box(PengBox::String(right))) => {
                Ok(left > right)
            }

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

            (Self::Box(PengBox::String(left)), Self::Box(PengBox::String(right))) => {
                Ok(left >= right)
            }

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

            (Self::Box(PengBox::String(left)), Self::Box(PengBox::String(right))) => {
                Ok(left < right)
            }

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

            (Self::Box(PengBox::String(left)), Self::Box(PengBox::String(right))) => {
                Ok(left <= right)
            }

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

        fn cell_to_value(cell: &PengCell) -> PengValue {
            PengValue::Cell(cell.clone())
        }

        fn value_impl_custom_type(value: &PengValue, custom_type: &PengCustomType) -> bool {
            match value {
                PengValue::Box(PengBox::Object(obj)) => custom_type
                    .fields
                    .keys()
                    .all(|name| obj.fields.contains_key(name)),

                _ => false,
            }
        }

        match (self, target_type) {
            // Any
            (value, PengType::Any) => Ok(value),

            // Same primitive
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

            // Int
            (PengValue::Cell(PengCell::Int(value)), PengType::Uint) => usize::try_from(value)
                .map(|value| PengValue::Cell(PengCell::Uint(value)))
                .map_err(|_| PengError::InvalidConversion { from, to }),
            (PengValue::Cell(PengCell::Int(value)), PengType::Byte) => u8::try_from(value)
                .map(|value| PengValue::Cell(PengCell::Byte(value)))
                .map_err(|_| PengError::InvalidConversion { from, to }),
            (PengValue::Cell(PengCell::Int(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value as f32)))
            }
            (PengValue::Cell(PengCell::Int(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            // Uint
            (PengValue::Cell(PengCell::Uint(value)), PengType::Int) => isize::try_from(value)
                .map(|value| PengValue::Cell(PengCell::Int(value)))
                .map_err(|_| PengError::InvalidConversion { from, to }),
            (PengValue::Cell(PengCell::Uint(value)), PengType::Byte) => u8::try_from(value)
                .map(|value| PengValue::Cell(PengCell::Byte(value)))
                .map_err(|_| PengError::InvalidConversion { from, to }),
            (PengValue::Cell(PengCell::Uint(value)), PengType::Float32) => {
                Ok(PengValue::Cell(PengCell::Float32(value as f32)))
            }
            (PengValue::Cell(PengCell::Uint(value)), PengType::Float64) => {
                Ok(PengValue::Cell(PengCell::Float64(value as f64)))
            }

            // Byte
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

            // Float32
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

            // Float64
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

            // Same heap type
            (PengValue::Box(PengBox::String(value)), PengType::String) => {
                Ok(PengValue::Box(PengBox::String(value)))
            }
            (PengValue::Box(PengBox::Object(value)), PengType::Object) => {
                Ok(PengValue::Box(PengBox::Object(value)))
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

            // Vector with inner type check
            (PengValue::Box(PengBox::Vector(vector)), PengType::Vector(inner_type)) => {
                if inner_type.equals(&PengType::Any) {
                    return Ok(PengValue::Box(PengBox::Vector(vector)));
                }

                for item in &vector.values {
                    let item_value = cell_to_value(item.value());

                    if item_value.convert((*inner_type).clone()).is_err() {
                        return Err(PengError::InvalidConversion { from, to });
                    }
                }

                Ok(PengValue::Box(PengBox::Vector(vector)))
            }

            // Custom structural cast
            (value, PengType::Custom(custom_type)) => {
                if value_impl_custom_type(&value, &custom_type) {
                    Ok(value)
                } else {
                    Err(PengError::InvalidConversion { from, to })
                }
            }

            // Any type -> String, except references/complex boxes
            (value, PengType::String) => {
                let converted = match value {
                    PengValue::Cell(PengCell::Nil) => "nil".to_string(),
                    PengValue::Cell(PengCell::Int(value)) => value.to_string(),
                    PengValue::Cell(PengCell::Uint(value)) => value.to_string(),
                    PengValue::Cell(PengCell::Byte(value)) => value.to_string(),
                    PengValue::Cell(PengCell::Float32(value)) => value.to_string(),
                    PengValue::Cell(PengCell::Float64(value)) => value.to_string(),
                    PengValue::Cell(PengCell::Bool(value)) => value.to_string(),

                    PengValue::Cell(PengCell::Reference(_)) => {
                        return Err(PengError::InvalidConversion { from, to });
                    }

                    PengValue::Box(PengBox::String(value)) => value,

                    PengValue::Box(
                        PengBox::Object(_)
                        | PengBox::Vector(_)
                        | PengBox::Type(_)
                        | PengBox::Module(_)
                        | PengBox::Thread(_)
                        | PengBox::Function(_)
                        | PengBox::Operation(_),
                    ) => {
                        return Err(PengError::InvalidConversion { from, to });
                    }
                };

                Ok(PengValue::Box(PengBox::String(converted)))
            }

            _ => Err(PengError::InvalidConversion { from, to }),
        }
    }
}
