use std::collections::HashMap;
use std::rc::Rc;

use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengUnit {
    init: Option<PengHeapPtr>,
    globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
    custom_access: HashMap<PengNamePoolPtr, PengNativeFunction>,
    custom_add: Option<PengNativeFunction>,
    custom_subtract: Option<PengNativeFunction>,
    custom_multiply: Option<PengNativeFunction>,
    custom_divide: Option<PengNativeFunction>,
    custom_power: Option<PengNativeFunction>,
    custom_remainder: Option<PengNativeFunction>,
}

impl PengUnit {
    pub fn new(
        init: PengHeapPtr,
        globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
        custom_access: HashMap<PengNamePoolPtr, PengNativeFunction>,
        custom_add: Option<PengNativeFunction>,
        custom_subtract: Option<PengNativeFunction>,
        custom_multiply: Option<PengNativeFunction>,
        custom_divide: Option<PengNativeFunction>,
        custom_power: Option<PengNativeFunction>,
        custom_remainder: Option<PengNativeFunction>,
    ) -> Self {
        Self {
            init: Some(init),
            globals,
            custom_access,
            custom_add,
            custom_subtract,
            custom_multiply,
            custom_divide,
            custom_power,
            custom_remainder,
        }
    }

    pub fn empty(init: PengHeapPtr) -> Self {
        Self {
            init: Some(init),
            globals: HashMap::new(),
            custom_access: HashMap::new(),
            custom_add: None,
            custom_subtract: None,
            custom_multiply: None,
            custom_divide: None,
            custom_power: None,
            custom_remainder: None,
        }
    }

    pub fn library() -> Self {
        Self {
            init: None,
            globals: HashMap::new(),
            custom_access: HashMap::new(),
            custom_add: None,
            custom_subtract: None,
            custom_multiply: None,
            custom_divide: None,
            custom_power: None,
            custom_remainder: None,
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

    pub fn insert_global(&mut self, name: PengNamePoolPtr, value: PengBindedHeapPtr) {
        self.globals.insert(name, value);
    }

    pub fn use_unit(&mut self, unit: &PengUnit) {
        for (name, value) in unit.globals.iter() {
            self.globals.insert(*name, value.clone());
        }

        for (name, value) in unit.custom_access.iter() {
            self.custom_access.insert(*name, value.clone());
        }

        if let Some(custom_add) = unit.custom_add.as_ref() {
            self.custom_add = Some(custom_add.clone());
        }

        if let Some(custom_subtract) = unit.custom_subtract.as_ref() {
            self.custom_subtract = Some(custom_subtract.clone());
        }

        if let Some(custom_multiply) = unit.custom_multiply.as_ref() {
            self.custom_multiply = Some(custom_multiply.clone());
        }

        if let Some(custom_divide) = unit.custom_divide.as_ref() {
            self.custom_divide = Some(custom_divide.clone());
        }

        if let Some(custom_power) = unit.custom_power.as_ref() {
            self.custom_power = Some(custom_power.clone());
        }

        if let Some(custom_remainder) = unit.custom_remainder.as_ref() {
            self.custom_remainder = Some(custom_remainder.clone());
        }
    }

    pub fn custom_access(&self) -> &HashMap<PengNamePoolPtr, PengNativeFunction> {
        &self.custom_access
    }
    pub fn custom_access_mut(&mut self) -> &mut HashMap<PengNamePoolPtr, PengNativeFunction> {
        &mut self.custom_access
    }

    pub fn custom_add(&self) -> Option<&PengNativeFunction> {
        self.custom_add.as_ref()
    }
    pub fn custom_add_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_add
    }

    pub fn custom_subtract(&self) -> Option<&PengNativeFunction> {
        self.custom_subtract.as_ref()
    }
    pub fn custom_subtract_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_subtract
    }

    pub fn custom_multiply(&self) -> Option<&PengNativeFunction> {
        self.custom_multiply.as_ref()
    }
    pub fn custom_multiply_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_multiply
    }

    pub fn custom_divide(&self) -> Option<&PengNativeFunction> {
        self.custom_divide.as_ref()
    }
    pub fn custom_divide_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_divide
    }

    pub fn custom_power(&self) -> Option<&PengNativeFunction> {
        self.custom_power.as_ref()
    }
    pub fn custom_power_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_power
    }

    pub fn custom_remainder(&self) -> Option<&PengNativeFunction> {
        self.custom_remainder.as_ref()
    }
    pub fn custom_remainder_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_remainder
    }

    pub fn register_custom_access<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        function: F,
    ) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        self.custom_access.insert(
            name_ptr,
            PengNativeFunction {
                call: Rc::new(function),
            },
        );
        Ok(())
    }

    pub fn register_custom_add<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_add = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_subtract<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_subtract = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_multiply<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_multiply = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_divide<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_divide = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_power<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_power = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_remainder<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_remainder = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_mutable_native_function<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        function: F,
    ) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(PengValue::Box(PengBox::Function(
            PengFunction::new_native(function),
        )));

        self.insert_global(name_ptr, PengBinded::Mutable(ptr));

        Ok(())
    }

    pub fn register_immutable_native_function<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        function: F,
    ) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(PengValue::Box(PengBox::Function(
            PengFunction::new_native(function),
        )));

        self.insert_global(name_ptr, PengBinded::Immutable(ptr));

        Ok(())
    }

    pub fn register_mutable_native_operation<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        operation: F,
    ) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(PengValue::Box(PengBox::Operation(
            PengOperation::new_native(operation),
        )));

        self.insert_global(name_ptr, PengBinded::Mutable(ptr));

        Ok(())
    }

    pub fn register_immutable_native_operation<F>(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        operation: F,
    ) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError> + 'static,
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

    pub fn retain_globals<F>(&mut self, f: F)
    where
        F: Fn(&PengNamePoolPtr, &mut PengBindedHeapPtr) -> bool,
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

    pub fn register_mutable_module(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        unit: &PengUnit,
    ) -> Result<(), PengError> {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let module_ptr = unit.create_module_heap(env);

        self.insert_global(name_ptr, PengBinded::Mutable(module_ptr));

        Ok(())
    }

    pub fn register_immutable_module(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        unit: &PengUnit,
    ) -> Result<(), PengError> {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let module_ptr = unit.create_module_heap(env);

        self.insert_global(name_ptr, PengBinded::Immutable(module_ptr));

        Ok(())
    }

    pub fn register_mutable_global(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        value: PengValue,
    ) -> Result<(), PengError> {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(value);

        self.insert_global(name_ptr, PengBinded::Mutable(ptr));

        Ok(())
    }

    pub fn register_immutable_global(
        &mut self,
        env: &mut PengEnv,
        name: &str,
        value: PengValue,
    ) -> Result<(), PengError> {
        let name_ptr = env.ensure_pooled_name_ptr(name.to_string());
        let ptr = env.create_heap_value(value);

        self.insert_global(name_ptr, PengBinded::Immutable(ptr));

        Ok(())
    }
}

fn binded_heap_ptr_to_cell(value: &PengBindedHeapPtr) -> PengBindedCell {
    match value {
        PengBinded::Mutable(ptr) => PengBinded::Mutable(PengCell::Reference(*ptr)),

        PengBinded::Immutable(ptr) => PengBinded::Immutable(PengCell::Reference(*ptr)),
    }
}
