use crate::core::PengBox;
use crate::core::binding::*;
use crate::core::cell::*;
use crate::core::env::*;
use crate::core::utils::*;
use crate::core::value::*;
use crate::core::*;

pub struct PengNativeFunctionCallContext<'a> {
    env: &'a mut PengEnv,
    thread: PengHeapPtr,
    unit: &'a PengUnit,
    args: Vec<PengBindedCell>,
}

impl<'a> PengNativeFunctionCallContext<'a> {
    pub fn new(
        env: &'a mut PengEnv,
        thread: PengHeapPtr,
        unit: &'a PengUnit,
        args: Vec<PengBindedCell>,
    ) -> Self {
        Self {
            env,
            thread,
            unit,
            args,
        }
    }

    pub fn env(&self) -> &PengEnv {
        self.env
    }

    pub fn env_mut(&mut self) -> &mut PengEnv {
        self.env
    }

    pub fn unit(&self) -> &PengUnit {
        self.unit
    }

    pub fn thread(&self) -> PengHeapPtr {
        self.thread
    }

    pub fn set_current_thread_state(&mut self, state: PengThreadState) -> Result<(), PengError> {
        match self.env.get_heap_mut(self.thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                thread.state = state;
                Ok(())
            }

            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(self.thread)),
        }
    }

    pub fn yield_now(&mut self) -> Result<(), PengError> {
        match self.env.get_heap_mut(self.thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                thread.quantum = 0;
                Ok(())
            }

            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(self.thread)),
        }
    }

    pub fn get_arg_cell(&self, arg: usize) -> Option<&PengBindedCell> {
        self.args.get(arg)
    }

    pub fn get_arg_cell_mut(&mut self, arg: usize) -> Option<&mut PengBindedCell> {
        self.args.get_mut(arg)
    }

    pub fn get_value(&self, ptr: PengHeapPtr) -> Option<&PengValue> {
        self.env.get_heap(ptr)
    }

    pub fn get_value_mut(&mut self, ptr: PengHeapPtr) -> Option<&mut PengValue> {
        self.env.get_heap_mut(ptr)
    }

    pub fn get_arg_value(&self, arg: usize) -> Option<PengBinded<&PengValue>> {
        match self.args.get(arg) {
            Some(PengBinded::Mutable(PengCell::Reference(ptr))) => {
                self.env.get_heap(*ptr).map(PengBinded::Mutable)
            }

            Some(PengBinded::Immutable(PengCell::Reference(ptr))) => {
                self.env.get_heap(*ptr).map(PengBinded::Immutable)
            }

            _ => None,
        }
    }

    pub fn get_arg_value_mut(&mut self, arg: usize) -> Option<PengBinded<&mut PengValue>> {
        match self.args.get(arg) {
            Some(PengBinded::Mutable(PengCell::Reference(ptr))) => {
                self.env.get_heap_mut(*ptr).map(PengBinded::Mutable)
            }

            _ => None,
        }
    }

    pub fn create_value(&mut self, value: PengValue) -> PengHeapPtr {
        self.env.create_heap_value(value)
    }

    pub fn create_box(&mut self, value: PengBox) -> PengHeapPtr {
        self.env.create_heap_value(PengValue::Box(value))
    }

    pub fn create_box_cell(&mut self, value: PengBox) -> PengCell {
        PengCell::Reference(self.create_box(value))
    }

    pub fn create_mutable_box_binded_cell(&mut self, value: PengBox) -> PengBindedCell {
        PengBinded::Mutable(self.create_box_cell(value))
    }

    pub fn create_immutable_box_binded_cell(&mut self, value: PengBox) -> PengBindedCell {
        PengBinded::Immutable(self.create_box_cell(value))
    }

    pub fn load_penguin_function_from_source(
        &mut self,
        source: &str,
        function_name: &str,
        position_id: usize,
    ) -> Result<PengHeapPtr, PengError> {
        let using_unit = (*self.unit).clone();

        match self.env.load_function_from_source_using(
            source,
            &using_unit,
            function_name,
            position_id,
        ) {
            Ok(function) => Ok(function),
            Err(e) => Err(e),
        }
    }

    pub fn run_penguin_function_from_source(
        &mut self,
        source: &str,
        function_name: &str,
        args: Vec<PengBindedCell>,
        position_id: usize,
    ) -> Result<PengBindedCell, PengError> {
        let using_unit = (*self.unit).clone();

        match self.env.run_function_from_source_using(
            source,
            &using_unit,
            function_name,
            args,
            position_id,
        ) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }
}

pub struct PengNativeOperationCallContext<'a> {
    env: &'a mut PengEnv,
    thread: PengHeapPtr,
    unit: &'a PengUnit,
    left: PengBindedCell,
    right: PengBindedCell,
}

impl<'a> PengNativeOperationCallContext<'a> {
    pub fn new(
        env: &'a mut PengEnv,
        thread: PengHeapPtr,
        unit: &'a PengUnit,
        left: PengBindedCell,
        right: PengBindedCell,
    ) -> Self {
        Self {
            env,
            thread,
            unit,
            left,
            right,
        }
    }

    pub fn env(&self) -> &PengEnv {
        self.env
    }

    pub fn env_mut(&mut self) -> &mut PengEnv {
        self.env
    }

    pub fn unit(&self) -> &PengUnit {
        self.unit
    }

    pub fn thread(&self) -> PengHeapPtr {
        self.thread
    }

    pub fn set_current_thread_state(&mut self, state: PengThreadState) -> Result<(), PengError> {
        match self.env.get_heap_mut(self.thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                thread.state = state;
                Ok(())
            }

            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(self.thread)),
        }
    }

    pub fn yield_now(&mut self) -> Result<(), PengError> {
        match self.env.get_heap_mut(self.thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                thread.quantum = 0;
                Ok(())
            }

            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(self.thread)),
        }
    }

    pub fn get_left_cell(&self) -> &PengBindedCell {
        &self.left
    }

    pub fn get_left_cell_mut(&mut self) -> &mut PengBindedCell {
        &mut self.left
    }

    pub fn get_right_cell(&self) -> &PengBindedCell {
        &self.right
    }

    pub fn get_right_cell_mut(&mut self) -> &mut PengBindedCell {
        &mut self.right
    }

    pub fn get_value(&self, ptr: PengHeapPtr) -> Option<&PengValue> {
        self.env.get_heap(ptr)
    }

    pub fn get_value_mut(&mut self, ptr: PengHeapPtr) -> Option<&mut PengValue> {
        self.env.get_heap_mut(ptr)
    }

    pub fn get_left_value(&self) -> Option<PengBinded<&PengValue>> {
        match &self.left {
            PengBinded::Mutable(PengCell::Reference(ptr)) => {
                self.env.get_heap(*ptr).map(PengBinded::Mutable)
            }

            PengBinded::Immutable(PengCell::Reference(ptr)) => {
                self.env.get_heap(*ptr).map(PengBinded::Immutable)
            }

            _ => None,
        }
    }

    pub fn get_left_value_mut(&mut self) -> Option<PengBinded<&mut PengValue>> {
        match &self.left {
            PengBinded::Mutable(PengCell::Reference(ptr)) => {
                self.env.get_heap_mut(*ptr).map(PengBinded::Mutable)
            }

            _ => None,
        }
    }

    pub fn get_right_value(&self) -> Option<PengBinded<&PengValue>> {
        match &self.right {
            PengBinded::Mutable(PengCell::Reference(ptr)) => {
                self.env.get_heap(*ptr).map(PengBinded::Mutable)
            }

            PengBinded::Immutable(PengCell::Reference(ptr)) => {
                self.env.get_heap(*ptr).map(PengBinded::Immutable)
            }

            _ => None,
        }
    }

    pub fn get_right_value_mut(&mut self) -> Option<PengBinded<&mut PengValue>> {
        match &self.right {
            PengBinded::Mutable(PengCell::Reference(ptr)) => {
                self.env.get_heap_mut(*ptr).map(PengBinded::Mutable)
            }

            _ => None,
        }
    }

    pub fn create_value(&mut self, value: PengValue) -> PengHeapPtr {
        self.env.create_heap_value(value)
    }

    pub fn create_box(&mut self, value: PengBox) -> PengHeapPtr {
        self.env.create_heap_value(PengValue::Box(value))
    }

    pub fn create_box_cell(&mut self, value: PengBox) -> PengCell {
        PengCell::Reference(self.create_box(value))
    }

    pub fn create_mutable_box_binded_cell(&mut self, value: PengBox) -> PengBindedCell {
        PengBinded::Mutable(self.create_box_cell(value))
    }

    pub fn create_immutable_box_binded_cell(&mut self, value: PengBox) -> PengBindedCell {
        PengBinded::Immutable(self.create_box_cell(value))
    }

    pub fn load_penguin_function_from_source(
        &mut self,
        source: &str,
        function_name: &str,
        position_id: usize,
    ) -> Result<PengHeapPtr, PengError> {
        let using_unit = (*self.unit).clone();

        match self.env.load_function_from_source_using(
            source,
            &using_unit,
            function_name,
            position_id,
        ) {
            Ok(function) => Ok(function),
            Err(e) => Err(e),
        }
    }

    pub fn run_penguin_function_from_source(
        &mut self,
        source: &str,
        function_name: &str,
        args: Vec<PengBindedCell>,
        position_id: usize,
    ) -> Result<PengBindedCell, PengError> {
        let using_unit = (*self.unit).clone();

        match self.env.run_function_from_source_using(
            source,
            &using_unit,
            function_name,
            args,
            position_id,
        ) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }
}
