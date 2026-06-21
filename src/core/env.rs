use std::collections::HashMap;

use crate::utils::*;
use crate::value::*;
use crate::cell::*;
use crate::vector::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengValuePtr>,
    values: HashMap<PengValuePtr, PengValue>,
    name_pool: HashMap<PengNamePoolPtr, String>,
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
        self.name_pool
            .keys()
            .max()
            .map(|v| v + 1)
            .unwrap_or(0)
    }

    fn next_value_ptr(&self) -> PengValuePtr {
        self.values
            .keys()
            .max()
            .map(|v| v + 1)
            .unwrap_or(0)
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

    pub fn create_value(&mut self, value: PengValue) -> PengValuePtr {
        let value_ptr = self.next_value_ptr();
        self.values.insert(value_ptr, value);
        value_ptr
    }

    pub fn create_vector_from_cells(
        &mut self,
        cells: impl IntoIterator<Item = PengCell>,
    ) -> PengValuePtr {
        let mut vector = PengVector::new();

        for cell in cells {
            let value_ptr = match cell {
                PengCell::Reference(value_ptr) => value_ptr,
                PengCell::Nil => self.create_value(PengValue::Nil),
                PengCell::Int(value) => self.create_value(PengValue::Int(value)),
                PengCell::Uint(value) => self.create_value(PengValue::Uint(value)),
                PengCell::Float32(value) => {
                    self.create_value(PengValue::Float32(value))
                }
                PengCell::Float64(value) => {
                    self.create_value(PengValue::Float64(value))
                }
                PengCell::Byte(value) => self.create_value(PengValue::Byte(value)),
                PengCell::Bool(value) => self.create_value(PengValue::Bool(value)),
            };

            vector.push(value_ptr);
        }

        self.create_value(PengValue::Vector(vector))
    }

    pub fn create_global(&mut self, name: String) -> PengValuePtr {
        let name_ptr = self.get_pooled_name(name);

        if let Some(value_ptr) = self.globals.get(&name_ptr) {
            return *value_ptr;
        }

        let value_ptr = self.create_value(PengValue::Nil);
        self.globals.insert(name_ptr, value_ptr);

        value_ptr
    }

    pub fn set_global(
        &mut self,
        name: String,
        value_ptr: PengValuePtr,
    ) {
        let name_ptr = self.get_pooled_name(name);
        self.globals.insert(name_ptr, value_ptr);
    }

    pub fn get_global(&self, name: &str) -> Option<PengValuePtr> {
        let name_ptr = self
            .name_pool
            .iter()
            .find_map(|(name_ptr, pooled_name)| {
                if pooled_name == name {
                    Some(*name_ptr)
                } else {
                    None
                }
            })?;

        self.globals.get(&name_ptr).copied()
    }

    pub fn get_value(
        &self,
        value_ptr: PengValuePtr,
    ) -> Option<&PengValue> {
        self.values.get(&value_ptr)
    }

    pub fn get_value_mut(
        &mut self,
        value_ptr: PengValuePtr,
    ) -> Option<&mut PengValue> {
        self.values.get_mut(&value_ptr)
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
}
