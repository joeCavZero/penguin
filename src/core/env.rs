use std::collections::HashMap;

use crate::utils::*;
use crate::value::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengValuePtr>,
    values: Vec<PengValue>,
    name_pool: Vec<String>,
    name_lookup: HashMap<String, PengNamePoolPtr>,
}

impl PengEnv {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            values: Vec::new(),
            name_pool: Vec::new(),
            name_lookup: HashMap::new(),
        }
    }

    pub fn get_pooled_name(
        &mut self,
        name: String,
    ) -> PengNamePoolPtr {
        match self.name_lookup.get(&name) {
            Some(name_ptr) => return *name_ptr,
            None => {}
        }

        let name_ptr = self.name_pool.len();
        self.name_pool.push(name.clone());
        self.name_lookup.insert(name, name_ptr);
        name_ptr
    }

    pub fn create_value(
        &mut self,
        value: PengValue,
    ) -> PengValuePtr {
        let value_ptr = self.values.len();
        self.values.push(value);
        value_ptr
    }

    pub fn create_global(
        &mut self,
        name: String,
    ) -> PengValuePtr {
        let name_ptr = self.get_pooled_name(name);

        match self.globals.get(&name_ptr) {
            Some(value_ptr) => return *value_ptr,
            None => {}
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

    pub fn get_global(
        &self,
        name: &str,
    ) -> Option<PengValuePtr> {
        match self.name_lookup.get(name) {
            Some(name_ptr) => match self.globals.get(name_ptr) {
                Some(value_ptr) => Some(*value_ptr),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_value(
        &self,
        value_ptr: PengValuePtr,
    ) -> Option<&PengValue> {
        self.values.get(value_ptr)
    }

    pub fn get_value_mut(
        &mut self,
        value_ptr: PengValuePtr,
    ) -> Option<&mut PengValue> {
        self.values.get_mut(value_ptr)
    }
}

impl Default for PengEnv {
    fn default() -> Self {
        Self::new()
    }
}
