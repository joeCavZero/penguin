use std::collections::HashMap;

use crate::binding::*;
use crate::cell::*;
use crate::error::*;
use crate::utils::*;
use crate::value::*;
use crate::vector::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengValuePtr>,
    pub values: HashMap<PengValuePtr, PengBindedValue>,
    pub name_pool: HashMap<PengNamePoolPtr, String>,
}

impl PengEnv {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            values: HashMap::new(),
            name_pool: HashMap::new(),
        }
    }

    fn next_name_ptr(&self) -> PengNamePoolPtr {
        self.name_pool.keys().max().map(|v| v + 1).unwrap_or(0)
    }

    fn next_value_ptr(&self) -> PengValuePtr {
        self.values.keys().max().map(|v| v + 1).unwrap_or(0)
    }

    pub fn get_pooled_name(&mut self, name: String) -> PengNamePoolPtr {
        for (name_ptr, pooled_name) in &self.name_pool {
            if pooled_name == &name {
                return *name_ptr;
            }
        }

        let name_ptr = self.next_name_ptr();
        self.name_pool.insert(name_ptr, name);
        name_ptr
    }

    pub fn get_name(&self, name_ptr: PengNamePoolPtr) -> Option<&String> {
        self.name_pool.get(&name_ptr)
    }

    pub fn create_value(&mut self, value: PengBindedValue) -> PengValuePtr {
        let value_ptr = self.next_value_ptr();
        self.values.insert(value_ptr, value);
        value_ptr
    }

    pub fn create_vector_from_cells(
        &mut self,
        cells: impl IntoIterator<Item = PengBindedCell>,
    ) -> Result<PengValuePtr, PengError> {
        let mut vector = PengVector::new();

        for cell in cells {
            let value_ptr = match cell {
                PengBinded::Mutable(cell) => self.create_cell_value(cell, false),

                PengBinded::Immutable(cell) => self.create_cell_value(cell, true),

                PengBinded::UninitializedImmutable => {
                    return Err(PengError::new_message("cannot create vector from uninitialized immutable cell".to_string()));
                }
            };

            vector.push(value_ptr);
        }

        Ok(self.create_value(PengBinded::Mutable(PengValue::Vector(vector))))
    }

    pub fn create_cell_value(&mut self, cell: PengCell, immutable: bool) -> PengValuePtr {
        match cell {
            PengCell::Reference(value_ptr) => value_ptr,

            PengCell::Nil => {
                let value = PengValue::Nil;
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Int(v) => {
                let value = PengValue::Int(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Uint(v) => {
                let value = PengValue::Uint(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Float32(v) => {
                let value = PengValue::Float32(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Float64(v) => {
                let value = PengValue::Float64(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Byte(v) => {
                let value = PengValue::Byte(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Bool(v) => {
                let value = PengValue::Bool(v);
                if immutable {
                    self.create_value(PengBinded::Immutable(value))
                } else {
                    self.create_value(PengBinded::Mutable(value))
                }
            }
        }
    }

    pub fn create_global(&mut self, name: String) -> PengValuePtr {
        let name_ptr = self.get_pooled_name(name);

        if let Some(value_ptr) = self.globals.get(&name_ptr) {
            return *value_ptr;
        }

        let value_ptr = self.create_value(PengBinded::Mutable(PengValue::Nil));
        self.globals.insert(name_ptr, value_ptr);

        value_ptr
    }

    pub fn set_global(&mut self, name: String, value_ptr: PengValuePtr) {
        let name_ptr = self.get_pooled_name(name);
        self.globals.insert(name_ptr, value_ptr);
    }

    pub fn get_global(&self, name: &str) -> Option<PengValuePtr> {
        let name_ptr = match self.name_pool.iter().find_map(|(name_ptr, pooled_name)| {
            if pooled_name == name {
                Some(*name_ptr)
            } else {
                None
            }
        }) {
            Some(v) => v,
            None => return None,
        };

        self.globals.get(&name_ptr).copied()
    }

    pub fn get_value(&self, value_ptr: PengValuePtr) -> Option<&PengValue> {
        match self.values.get(&value_ptr) {
            Some(PengBinded::Mutable(value)) => Some(value),
            Some(PengBinded::Immutable(value)) => Some(value),
            Some(PengBinded::UninitializedImmutable) => None,
            None => None,
        }
    }

    pub fn get_value_mut(&mut self, value_ptr: PengValuePtr) -> Option<&mut PengValue> {
        match self.values.get_mut(&value_ptr) {
            Some(PengBinded::Mutable(value)) => Some(value),
            _ => None,
        }
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.globals == rhs.globals
            && self.name_pool == rhs.name_pool
            && self.values.len() == rhs.values.len()
            && self.values.iter().all(|(value_ptr, value)| {
                rhs.values
                    .get(value_ptr)
                    .is_some_and(|rhs_value| value.equals(rhs_value))
            })
    }

    pub fn set_value(
        &mut self,
        value_ptr: PengValuePtr,
        value: PengValue,
    ) -> Result<(), PengError> {
        match self.values.get_mut(&value_ptr) {
            Some(PengBinded::Mutable(current)) => {
                *current = value;
            }

            Some(current @ PengBinded::UninitializedImmutable) => {
                *current = PengBinded::Immutable(value);
            }

            Some(PengBinded::Immutable(_)) => {
                return Err(PengError::new_message(
                    "cannot set immutable value".to_string(),
                ));
            }

            None => {
                return Err(PengError::new_message("value not found".to_string()));
            }
        }

        Ok(())
    }
}
