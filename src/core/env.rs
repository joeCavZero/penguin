use std::collections::HashMap;

use crate::colour::*;
use crate::state::*;
use crate::binding::*;
use crate::cell::*;
use crate::error::*;
use crate::utils::*;
use crate::value::*;
use crate::vector::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengHeapPtr>,
    pub heap: HashMap<PengHeapPtr, PengColouredBindedStatedValue>,
    pub name_pool: HashMap<PengNamePoolPtr, String>,
}

impl PengEnv {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            heap: HashMap::new(),
            name_pool: HashMap::new(),
        }
    }

    fn next_name_ptr(&self) -> PengNamePoolPtr {
        self.name_pool.keys().max().map(|v| v + 1).unwrap_or(0)
    }

    fn next_heap_ptr(&self) -> PengHeapPtr {
        self.heap.keys().max().map(|v| v + 1).unwrap_or(0)
    }

    pub fn ensure_pooled_name_ptr(&mut self, name: String) -> PengNamePoolPtr {
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

    pub fn create_binded_stated_heap(&mut self, value: PengBindedStatedValue) -> PengHeapPtr {
        let value_ptr = self.next_heap_ptr();
        let coloured = PengColoured::new(value);
        self.heap.insert(value_ptr, coloured);
        value_ptr
    }

    pub fn create_mutable_initialized_vector_from_binded_stated_cells(
        &mut self,
        cells: impl IntoIterator<Item = PengBindedStatedCell>,
    ) -> Result<PengHeapPtr, PengError> {
        let mut vector = PengVector::new();

        for cell in cells {
            vector.push(cell);
        }

        Ok(
            self.create_binded_stated_heap(
                PengBinded::Mutable(
                    PengStated::Initialized(PengValue::Vector(vector)),
                )
            )
        )
    }
    
    /*
    pub fn create_stated_cell_value(&mut self, cell: PengStatedCell) -> PengHeapPtr {
        match cell {
            PengCell::Reference(value_ptr) => value_ptr,

            PengCell::Nil => {
                let value = PengValue::Nil;
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Int(v) => {
                let value = PengValue::Int(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Uint(v) => {
                let value = PengValue::Uint(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Float32(v) => {
                let value = PengValue::Float32(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Float64(v) => {
                let value = PengValue::Float64(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Byte(v) => {
                let value = PengValue::Byte(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }

            PengCell::Bool(v) => {
                let value = PengValue::Bool(v);
                if immutable {
                    self.create_binded_stated_value(PengBinded::Immutable(value))
                } else {
                    self.create_binded_stated_value(PengBinded::Mutable(value))
                }
            }
        }
    }
     */

    pub fn ensure_mutable_uninitialized_global(&mut self, name: String) -> PengHeapPtr {
        let name_ptr = self.ensure_pooled_name_ptr(name);

        if let Some(value_ptr) = self.globals.get(&name_ptr) {
            return *value_ptr;
        }

        let heap_ptr = self.create_binded_stated_heap(
            PengBinded::Mutable(
                PengStated::Uninitialized,
            )
        );

        self.globals.insert(name_ptr, heap_ptr);

        heap_ptr
    }

    pub fn set_global(&mut self, name: String, value_ptr: PengHeapPtr) {
        let name_ptr = self.ensure_pooled_name_ptr(name);
        self.globals.insert(name_ptr, value_ptr);
    }

    pub fn get_global_ptr_by_name_str(&self, name: &str) -> Option<PengHeapPtr> {
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

    pub fn get_heap(&self, value_ptr: PengHeapPtr) -> Option<&PengColouredBindedStatedValue> {
        self.heap.get(&value_ptr)
    }

    pub fn get_heap_mut(
        &mut self,
        value_ptr: PengHeapPtr,
    ) -> Option<&mut PengColouredBindedStatedValue> {
        self.heap.get_mut(&value_ptr)
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        self.globals == rhs.globals
            && self.name_pool == rhs.name_pool
            && self.heap.len() == rhs.heap.len()
            && self.heap.iter().all(|(value_ptr, value)| {
                rhs.heap
                    .get(value_ptr)
                    .is_some_and(|rhs_value| value.equals(rhs_value))
            })
    }

    pub fn assign_heap(
        &mut self,
        heap_ptr: PengHeapPtr,
        heap: PengValue,
    ) -> Result<(), PengError> {
        let coloured = self
            .heap
            .get_mut(&heap_ptr)
            .ok_or_else(|| PengError::new_message("value not found".to_string()))?;

        match &mut coloured.value {
            PengBinded::Mutable(current) => {
                *current = PengStated::Initialized(heap);
                Ok(())
            }

            PengBinded::Immutable(PengStated::Uninitialized) => {
                coloured.value = PengBinded::Immutable(PengStated::Initialized(heap));
                Ok(())
            }

            PengBinded::Immutable(_) => Err(PengError::new_message(
                "cannot set immutable value".to_string(),
            )),
        }
    }
}
