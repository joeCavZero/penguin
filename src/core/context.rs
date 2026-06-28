use crate::core::utils::*;
use crate::core::binding::*;
use crate::core::cell::*;
use crate::core::env::*;
use crate::core::value::*;

pub struct PengNativeFunctionCallContext<'a> {
    env: &'a mut PengEnv,
    args: Vec<PengBindedCell>,
}

impl<'a> PengNativeFunctionCallContext<'a> {
    pub fn new(env: &'a mut PengEnv, args: Vec<PengBindedCell>) -> Self {
        Self { env, args }
    }

    pub fn env(&self) -> &PengEnv {
        self.env
    }

    pub fn env_mut(&mut self) -> &mut PengEnv {
        self.env
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
}

pub struct PengNativeOperationCallContext<'a> {
    env: &'a mut PengEnv,
    left: PengBindedCell,
    right: PengBindedCell,
}

impl<'a> PengNativeOperationCallContext<'a> {
    pub fn new(env: &'a mut PengEnv, left: PengBindedCell, right: PengBindedCell) -> Self {
        Self { env, left, right }
    }

    pub fn env(&self) -> &PengEnv {
        self.env
    }

    pub fn env_mut(&mut self) -> &mut PengEnv {
        self.env
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
}