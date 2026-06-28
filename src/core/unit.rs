use std::collections::HashMap;

use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengUnit {
    init: Option<PengHeapPtr>,
    globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
}

impl PengUnit {
    pub fn new(
        init: PengHeapPtr,
        globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
    ) -> Self {
        Self {
            init: Some(init),
            globals,
        }
    }

    pub fn empty(init: PengHeapPtr) -> Self {
        Self {
            init: Some(init),
            globals: HashMap::new(),
        }
    }

    pub fn library() -> Self {
        Self {
            init: None,
            globals: HashMap::new(),
        }
    }

    pub fn init(&self) -> Option<PengHeapPtr> {
        self.init
    }

    pub fn require_init(&self) -> Result<PengHeapPtr, PengError> {
        match self.init {
            Some(init) => Ok(init),
            None => Err(PengError::InvalidState(
                "unit has no init function".to_string(),
            )),
        }
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

    pub fn use_unit(&mut self, unit: &PengUnit) {
        for (name, value) in unit.globals.iter() {
            self.globals.insert(*name, value.clone());
        }
    }

    pub fn register_native_function<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        function: F,
    ) -> Result<(), PengError>
    where
        F: FnMut(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(PengValue::Box(PengBox::Function(
            PengFunction::new_native(function),
        )));

        self.insert_global(name_ptr, PengBinded::Immutable(ptr));

        Ok(())
    }

    pub fn register_native_operation<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        operation: F,
    ) -> Result<(), PengError>
    where
        F: FnMut(&mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError>
            + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(PengValue::Box(PengBox::Operation(
            PengOperation::new_native(operation),
        )));

        self.insert_global(name_ptr, PengBinded::Immutable(ptr));

        Ok(())
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
