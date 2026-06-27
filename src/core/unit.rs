use std::collections::HashMap;

use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengUnit {
    pub init: PengHeapPtr,
    pub globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
}

impl PengUnit {
    pub fn new(
        init: PengHeapPtr,
        globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
    ) -> Self {
        Self {
            init,
            globals,
        }
    }

    pub fn empty(init: PengHeapPtr) -> Self {
        Self {
            init,
            globals: HashMap::new(),
        }
    }

    pub fn init(&self) -> PengHeapPtr {
        self.init
    }

    pub fn globals(&self) -> &HashMap<PengNamePoolPtr, PengBindedHeapPtr> {
        &self.globals
    }

    pub fn globals_mut(&mut self) -> &mut HashMap<PengNamePoolPtr, PengBindedHeapPtr> {
        &mut self.globals
    }

    pub fn into_globals(self) -> HashMap<PengNamePoolPtr, PengBindedHeapPtr> {
        self.globals
    }

    pub fn get_global(&self, name: PengNamePoolPtr) -> Option<&PengBindedHeapPtr> {
        self.globals.get(&name)
    }

    pub fn get_global_ptr(&self, name: PengNamePoolPtr) -> Option<PengHeapPtr> {
        match self.globals.get(&name) {
            Some(value) => Some(*value.value()),
            None => None,
        }
    }

    pub fn insert_global(
        &mut self,
        name: PengNamePoolPtr,
        value: PengBindedHeapPtr,
    ) {
        self.globals.insert(name, value);
    }

    pub fn remove_global(&mut self, name: PengNamePoolPtr) -> Option<PengBindedHeapPtr> {
        self.globals.remove(&name)
    }

    pub fn retain_globals<F>(&mut self, mut f: F)
    where
        F: FnMut(&PengNamePoolPtr, &mut PengBindedHeapPtr) -> bool,
    {
        self.globals.retain(|name, value| f(name, value));
    }

    pub fn to_module(&self) -> PengModule {
        let mut module = PengModule::new_empty();

        for (name, value) in self.globals.iter() {
            module.members.insert(*name, binded_heap_ptr_to_cell(value));
        }

        module
    }

    pub fn create_module_value(&self) -> PengValue {
        PengValue::Box(PengBox::Module(self.to_module()))
    }

    pub fn create_module_heap(&self, env: &mut PengEnv) -> PengHeapPtr {
        env.create_heap_value(self.create_module_value())
    }
}

fn binded_heap_ptr_to_cell(value: &PengBindedHeapPtr) -> PengBindedCell {
    match value {
        PengBinded::Mutable(ptr) => {
            PengBinded::Mutable(PengCell::Reference(*ptr))
        }

        PengBinded::Immutable(ptr) => {
            PengBinded::Immutable(PengCell::Reference(*ptr))
        }
    }
}