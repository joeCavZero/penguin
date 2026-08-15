use std::collections::HashMap;
use std::rc::Rc;

use crate::core::*;

#[derive(Debug, Clone)]
pub struct PengUnit {
    init: Option<PengHeapPtr>,
    globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,

    custom_access: HashMap<PengNamePoolPtr, PengNativeFunction>,
    custom_call: Option<PengNativeFunction>,

    custom_add: Option<PengNativeFunction>,
    custom_subtract: Option<PengNativeFunction>,
    custom_multiply: Option<PengNativeFunction>,
    custom_divide: Option<PengNativeFunction>,
    custom_power: Option<PengNativeFunction>,
    custom_remainder: Option<PengNativeFunction>,
    custom_negate: Option<PengNativeFunction>,
    custom_concat: Option<PengNativeFunction>,
    custom_and: Option<PengNativeFunction>,
    custom_or: Option<PengNativeFunction>,
    custom_not: Option<PengNativeFunction>,
    custom_equals: Option<PengNativeFunction>,
    custom_not_equals: Option<PengNativeFunction>,
    custom_greater_than: Option<PengNativeFunction>,
    custom_greater_equals_than: Option<PengNativeFunction>,
    custom_less_than: Option<PengNativeFunction>,
    custom_less_equals_than: Option<PengNativeFunction>,
}

impl PengUnit {
    pub fn new(
        init: PengHeapPtr,
        globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
        custom_access: HashMap<PengNamePoolPtr, PengNativeFunction>,
        custom_call: Option<PengNativeFunction>,
        custom_add: Option<PengNativeFunction>,
        custom_subtract: Option<PengNativeFunction>,
        custom_multiply: Option<PengNativeFunction>,
        custom_divide: Option<PengNativeFunction>,
        custom_power: Option<PengNativeFunction>,
        custom_remainder: Option<PengNativeFunction>,
        custom_negate: Option<PengNativeFunction>,
        custom_concat: Option<PengNativeFunction>,
        custom_and: Option<PengNativeFunction>,
        custom_or: Option<PengNativeFunction>,
        custom_not: Option<PengNativeFunction>,
        custom_equals: Option<PengNativeFunction>,
        custom_not_equals: Option<PengNativeFunction>,
        custom_greater_than: Option<PengNativeFunction>,
        custom_greater_equals_than: Option<PengNativeFunction>,
        custom_less_than: Option<PengNativeFunction>,
        custom_less_equals_than: Option<PengNativeFunction>,
    ) -> Self {
        Self {
            init: Some(init),
            globals,
            custom_access,
            custom_call,
            custom_add,
            custom_subtract,
            custom_multiply,
            custom_divide,
            custom_power,
            custom_remainder,
            custom_negate,
            custom_concat,
            custom_and,
            custom_or,
            custom_not,
            custom_equals,
            custom_not_equals,
            custom_greater_than,
            custom_greater_equals_than,
            custom_less_than,
            custom_less_equals_than,
        }
    }

    pub fn empty(init: PengHeapPtr) -> Self {
        Self {
            init: Some(init),
            globals: HashMap::new(),
            custom_access: HashMap::new(),
            custom_call: None,
            custom_add: None,
            custom_subtract: None,
            custom_multiply: None,
            custom_divide: None,
            custom_power: None,
            custom_remainder: None,
            custom_negate: None,
            custom_concat: None,
            custom_and: None,
            custom_or: None,
            custom_not: None,
            custom_equals: None,
            custom_not_equals: None,
            custom_greater_than: None,
            custom_greater_equals_than: None,
            custom_less_than: None,
            custom_less_equals_than: None,
        }
    }

    pub fn library() -> Self {
        Self {
            init: None,
            globals: HashMap::new(),
            custom_access: HashMap::new(),
            custom_call: None,
            custom_add: None,
            custom_subtract: None,
            custom_multiply: None,
            custom_divide: None,
            custom_power: None,
            custom_remainder: None,
            custom_negate: None,
            custom_concat: None,
            custom_and: None,
            custom_or: None,
            custom_not: None,
            custom_equals: None,
            custom_not_equals: None,
            custom_greater_than: None,
            custom_greater_equals_than: None,
            custom_less_than: None,
            custom_less_equals_than: None,
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

        if let Some(custom_call) = unit.custom_call.as_ref() {
            self.custom_call = Some(custom_call.clone());
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

        if let Some(custom_negate) = unit.custom_negate.as_ref() {
            self.custom_negate = Some(custom_negate.clone());
        }

        if let Some(custom_concat) = unit.custom_concat.as_ref() {
            self.custom_concat = Some(custom_concat.clone());
        }

        if let Some(custom_and) = unit.custom_and.as_ref() {
            self.custom_and = Some(custom_and.clone());
        }

        if let Some(custom_or) = unit.custom_or.as_ref() {
            self.custom_or = Some(custom_or.clone());
        }

        if let Some(custom_not) = unit.custom_not.as_ref() {
            self.custom_not = Some(custom_not.clone());
        }

        if let Some(custom_equals) = unit.custom_equals.as_ref() {
            self.custom_equals = Some(custom_equals.clone());
        }

        if let Some(custom_not_equals) = unit.custom_not_equals.as_ref() {
            self.custom_not_equals = Some(custom_not_equals.clone());
        }

        if let Some(custom_greater_than) = unit.custom_greater_than.as_ref() {
            self.custom_greater_than = Some(custom_greater_than.clone());
        }

        if let Some(custom_greater_equals_than) = unit.custom_greater_equals_than.as_ref() {
            self.custom_greater_equals_than = Some(custom_greater_equals_than.clone());
        }

        if let Some(custom_less_than) = unit.custom_less_than.as_ref() {
            self.custom_less_than = Some(custom_less_than.clone());
        }

        if let Some(custom_less_equals_than) = unit.custom_less_equals_than.as_ref() {
            self.custom_less_equals_than = Some(custom_less_equals_than.clone());
        }
    }

    pub fn custom_access(&self) -> &HashMap<PengNamePoolPtr, PengNativeFunction> {
        &self.custom_access
    }
    pub fn custom_access_mut(&mut self) -> &mut HashMap<PengNamePoolPtr, PengNativeFunction> {
        &mut self.custom_access
    }

    pub fn custom_call(&self) -> Option<&PengNativeFunction> {
        self.custom_call.as_ref()
    }
    pub fn custom_call_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_call
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

    pub fn custom_negate(&self) -> Option<&PengNativeFunction> {
        self.custom_negate.as_ref()
    }
    pub fn custom_negate_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_negate
    }

    pub fn custom_concat(&self) -> Option<&PengNativeFunction> {
        self.custom_concat.as_ref()
    }
    pub fn custom_concat_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_concat
    }

    pub fn custom_and(&self) -> Option<&PengNativeFunction> {
        self.custom_and.as_ref()
    }
    pub fn custom_and_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_and
    }

    pub fn custom_or(&self) -> Option<&PengNativeFunction> {
        self.custom_or.as_ref()
    }
    pub fn custom_or_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_or
    }

    pub fn custom_not(&self) -> Option<&PengNativeFunction> {
        self.custom_not.as_ref()
    }
    pub fn custom_not_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_not
    }

    pub fn custom_equals(&self) -> Option<&PengNativeFunction> {
        self.custom_equals.as_ref()
    }
    pub fn custom_equals_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_equals
    }

    pub fn custom_not_equals(&self) -> Option<&PengNativeFunction> {
        self.custom_not_equals.as_ref()
    }
    pub fn custom_not_equals_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_not_equals
    }

    pub fn custom_greater_than(&self) -> Option<&PengNativeFunction> {
        self.custom_greater_than.as_ref()
    }
    pub fn custom_greater_than_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_greater_than
    }

    pub fn custom_greater_equals_than(&self) -> Option<&PengNativeFunction> {
        self.custom_greater_equals_than.as_ref()
    }
    pub fn custom_greater_equals_than_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_greater_equals_than
    }

    pub fn custom_less_than(&self) -> Option<&PengNativeFunction> {
        self.custom_less_than.as_ref()
    }
    pub fn custom_less_than_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_less_than
    }

    pub fn custom_less_equals_than(&self) -> Option<&PengNativeFunction> {
        self.custom_less_equals_than.as_ref()
    }
    pub fn custom_less_equals_than_mut(&mut self) -> &mut Option<PengNativeFunction> {
        &mut self.custom_less_equals_than
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

    pub fn register_custom_call<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_call = Some(PengNativeFunction {
            call: Rc::new(function),
        });

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

    pub fn register_custom_negate<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_negate = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_concat<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_concat = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_and<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_and = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_or<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_or = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_not<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_not = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_equals<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_equals = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_not_equals<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_not_equals = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_greater_than<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_greater_than = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_greater_equals_than<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_greater_equals_than = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_less_than<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_less_than = Some(PengNativeFunction {
            call: Rc::new(function),
        });

        Ok(())
    }

    pub fn register_custom_less_equals_than<F>(&mut self, function: F) -> Result<(), PengError>
    where
        F: Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> + 'static,
    {
        self.custom_less_equals_than = Some(PengNativeFunction {
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
