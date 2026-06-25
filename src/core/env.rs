use std::collections::HashMap;

use crate::binding::*;
use crate::cell::*;
use crate::colour::*;
use crate::error::*;
use crate::frame::*;
use crate::heap_value::*;
use crate::state::*;
use crate::thread::*;
use crate::utils::*;
use crate::value::*;
use crate::vector::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengBindedStatedCell>,
    pub heap: HashMap<PengHeapPtr, PengColouredHeapValue>,
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

    pub fn get_pooled_name(&self, name_ptr: PengNamePoolPtr) -> Option<&String> {
        self.name_pool.get(&name_ptr)
    }

    pub fn create_heap_value(&mut self, value: PengHeapValue) -> PengHeapPtr {
        let value_ptr = self.next_heap_ptr();
        self.heap.insert(value_ptr, PengColoured::new(value));
        value_ptr
    }

    pub fn create_vector_from_cells(
        &mut self,
        cells: impl IntoIterator<Item = PengBindedStatedCell>,
    ) -> Result<PengHeapPtr, PengError> {
        let mut vector = PengVector::new_empty();

        for cell in cells {
            vector.push(cell);
        }

        Ok(self.create_heap_value(PengHeapValue::Vector(vector)))
    }

    pub fn set_global(&mut self, name: String, cell: PengBindedStatedCell) {
        let name_ptr = self.ensure_pooled_name_ptr(name);
        self.globals.insert(name_ptr, cell);
    }

    pub fn get_global_by_name_str(&self, name: &str) -> Option<&PengBindedStatedCell> {
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

        self.globals.get(&name_ptr)
    }

    pub fn get_heap(&self, value_ptr: PengHeapPtr) -> Option<&PengHeapValue> {
        match self.heap.get(&value_ptr) {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    pub fn get_heap_mut(&mut self, value_ptr: PengHeapPtr) -> Option<&mut PengHeapValue> {
        match self.heap.get_mut(&value_ptr) {
            Some(v) => Some(&mut v.value),
            None => None,
        }
    }

    pub fn assign_heap(
        &mut self,
        heap_ptr: PengHeapPtr,
        value: PengHeapValue,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(heap_ptr) {
            Some(heap_value) => {
                *heap_value = value;
                return Ok(());
            }
            None => {
                return Err(PengError::HeapValueNotFound(heap_ptr));
            }
        };
    }

    pub fn create_thread(
        &mut self,
        procedure_ptr: PengHeapPtr,
        base: usize,
        params: Vec<PengBindedStatedCell>,
    ) -> PengHeapPtr {
        let t = PengHeapValue::Thread(PengThread::new(procedure_ptr, base, params));
        self.create_heap_value(t)
    }

    pub fn push_thread_binded_stated_cell(
        &mut self,
        thread_ptr: PengHeapPtr,
        cell: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread_ptr) {
            Ok(thread) => {
                thread.stack.push(cell);
                Ok(())
            }
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn get_thread_latest_binded_stated_cell(
        &self,
        thread_ptr: PengHeapPtr,
        offset: usize,
    ) -> Result<&PengBindedStatedCell, PengError> {
        match self.get_thread(thread_ptr) {
            Ok(thread) => {
                if thread.stack.len() <= offset {
                    return Err(PengError::StackUnderflow);
                }

                let index = thread.stack.len() - offset - 1;

                match thread.stack.get(index) {
                    Some(v) => Ok(v),
                    None => Err(PengError::StackUnderflow),
                }
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn get_thread_2_latests_binded_stated_cell_cloned(
        &self,
        thread_ptr: PengHeapPtr,
    ) -> Result<(PengBindedStatedCell, PengBindedStatedCell), PengError> {
        match self.get_heap(thread_ptr) {
            Some(h) => {
                if let PengHeapValue::Thread(thread) = h {
                    if thread.stack.len() < 2 {
                        return Err(PengError::StackUnderflow);
                    }

                    let rhs_index = thread.stack.len() - 1;
                    let lhs_index = thread.stack.len() - 2;

                    match (thread.stack.get(lhs_index), thread.stack.get(rhs_index)) {
                        (Some(lhs), Some(rhs)) => Ok((lhs.clone(), rhs.clone())),

                        _ => Err(PengError::StackUnderflow),
                    }
                } else {
                    Err(PengError::ExpectedThread)
                }
            }
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn get_thread_latest_n_binded_stated_cells_cloned(
        &self,
        thread_ptr: PengHeapPtr,
        n: usize,
    ) -> Result<Vec<PengBindedStatedCell>, PengError> {
        match self.get_thread(thread_ptr) {
            Ok(thread) => {
                if thread.stack.len() < n {
                    return Err(PengError::StackUnderflow);
                }

                Ok(thread.stack[thread.stack.len() - n..].to_vec())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn pop_thread_stack_n_times(
        &mut self,
        thread_ptr: PengHeapPtr,
        n: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread_ptr) {
            Ok(thread) => {
                for _ in 0..n {
                    match thread.stack.pop() {
                        Some(_) => {}
                        None => return Err(PengError::StackUnderflow),
                    }
                }

                Ok(())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn get_thread_local(
        &self,
        thread_ptr: PengHeapPtr,
        local: usize,
    ) -> Result<&PengBindedStatedCell, PengError> {
        match self.get_heap(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                let base = match thread.frames.last() {
                    Some(frame) => frame.base,
                    None => return Err(PengError::FrameNotFound),
                };

                match base.checked_add(local) {
                    Some(index) => match thread.stack.get(index) {
                        Some(cell) => Ok(cell),
                        None => Err(PengError::LocalNotFound(local)),
                    },

                    None => Err(PengError::InvalidFrame),
                }
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn set_thread_local(
        &mut self,
        thread_ptr: PengHeapPtr,
        local: usize,
        value: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                let base = match thread.frames.last() {
                    Some(frame) => frame.base,
                    None => return Err(PengError::FrameNotFound),
                };

                match base.checked_add(local) {
                    Some(index) => match thread.stack.get_mut(index) {
                        Some(cell) => {
                            *cell = value;
                            Ok(())
                        }

                        None => Err(PengError::LocalNotFound(local)),
                    },

                    None => Err(PengError::InvalidFrame),
                }
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn set_thread_program_counter(
        &mut self,
        thread_ptr: PengHeapPtr,
        target: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread_ptr) {
            Ok(thread) => match thread.frames.last_mut() {
                Some(frame) => {
                    frame.program_counter = target;
                    Ok(())
                }
                None => Err(PengError::FrameNotFound),
            },
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn push_thread_frame(
        &mut self,
        thread_ptr: PengHeapPtr,
        frame: PengFrame,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread_ptr) {
            Ok(thread) => {
                thread.frames.push(frame);
                Ok(())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn pop_thread_stack_at(
        &mut self,
        thread_ptr: PengHeapPtr,
        offset: usize,
    ) -> Result<PengBindedStatedCell, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                if thread.stack.len() <= offset {
                    return Err(PengError::StackUnderflow);
                }

                let index = thread.stack.len() - offset - 1;

                Ok(thread.stack.remove(index))
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn get_thread_stack_len(&self, thread_ptr: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_thread(thread_ptr) {
            Ok(thread) => Ok(thread.stack.len()),
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn execute_try_function_call(
        &mut self,
        thread_ptr: PengHeapPtr,
        args_count: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_stack_len(thread_ptr) {
            Ok(stack_len) => {
                if stack_len < args_count + 1 {
                    return Err(PengError::TooFewArguments {
                        expected: args_count + 1,
                        found: stack_len,
                    });
                }

                let function_index = stack_len - args_count - 1;

                match self.get_thread_latest_binded_stated_cell(thread_ptr, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        match self.pop_thread_stack_at(thread_ptr, args_count) {
                            Ok(_) => {
                                match self.push_thread_frame(
                                    thread_ptr,
                                    PengFrame::new_try(function_ptr, function_index, args_count),
                                ) {
                                    Ok(()) => Ok(()),
                                    Err(e) => {
                                        return Err(e.push(PengError::CannotCallValue(
                                            "failed while preparing try function call".to_string(),
                                        )));
                                    }
                                }
                            }

                            Err(e) => {
                                return Err(e.push(PengError::CannotCallValue(
                                    "failed while removing callee from stack".to_string(),
                                )));
                            }
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::CannotCallValue(
                            "failed while reading try function callee".to_string(),
                        )));
                    }
                }
            }

            Err(e) => return Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }

    pub fn get_thread_latest_frame(
        &self,
        thread_ptr: PengHeapPtr,
    ) -> Result<&PengFrame, PengError> {
        match self.get_heap(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => match thread.frames.last() {
                Some(frame) => Ok(frame),
                None => Err(PengError::FrameNotFound),
            },
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn pop_thread_frame(&mut self, thread_ptr: PengHeapPtr) -> Result<PengFrame, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => match thread.frames.pop() {
                Some(frame) => Ok(frame),
                None => Err(PengError::FrameNotFound),
            },
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn truncate_thread_stack(
        &mut self,
        thread_ptr: PengHeapPtr,
        len: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                thread.stack.truncate(len);
                Ok(())
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn get_thread_frames_len(&self, thread_ptr: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_heap(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => Ok(thread.frames.len()),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn recover_thread_try_error(&mut self, thread_ptr: PengHeapPtr) -> Result<bool, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                let mut try_frame = None;

                while let Some(frame) = thread.frames.pop() {
                    if frame.is_try {
                        try_frame = Some(frame);
                        break;
                    }
                }

                match try_frame {
                    Some(frame) => {
                        thread.stack.truncate(frame.base);

                        thread
                            .stack
                            .push(PengBinded::Mutable(PengStated::Initialized(PengCell::Nil)));

                        thread
                            .stack
                            .push(PengBinded::Mutable(PengStated::Initialized(
                                PengCell::Bool(false),
                            )));

                        Ok(true)
                    }

                    None => Ok(false),
                }
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    fn get_thread(&self, thread_ptr: PengHeapPtr) -> Result<&PengThread, PengError> {
        match self.get_heap(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => Ok(thread),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    fn get_thread_mut(&mut self, thread_ptr: PengHeapPtr) -> Result<&mut PengThread, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => Ok(thread),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread_ptr)),
        }
    }

    pub fn get_value_from_cell(&self, cell: PengCell) -> Result<PengValue, PengError> {
        match cell {
            PengCell::Reference(ptr) => match self.get_heap(ptr) {
                Some(value) => Ok(PengValue::Heap(value.clone())),
                None => Err(PengError::HeapValueNotFound(ptr)),
            },

            _ => Ok(PengValue::Cell(cell)),
        }
    }

    pub fn get_heap_value_from_cell(&self, cell: PengCell) -> Result<PengHeapValue, PengError> {
        match cell {
            PengCell::Reference(ptr) => match self.get_heap(ptr) {
                Some(value) => Ok(value.clone()),
                None => Err(PengError::HeapValueNotFound(ptr)),
            },

            _ => Err(PengError::ExpectedReference),
        }
    }

    pub fn get_cell_from_value(&mut self, value: PengValue) -> Result<PengCell, PengError> {
        match value {
            PengValue::Cell(cell) => Ok(cell),
            PengValue::Heap(value) => {
                let ptr = self.create_heap_value(value);
                Ok(PengCell::Reference(ptr))
            }
        }
    }
}
