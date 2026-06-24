use std::collections::HashMap;

use crate::binding::*;
use crate::cell::*;
use crate::colour::*;
use crate::error::*;
use crate::frame::*;
use crate::state::*;
use crate::thread::*;
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

    pub fn get_pooled_name(&self, name_ptr: PengNamePoolPtr) -> Option<&String> {
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
        let mut vector = PengVector::new_empty();

        for cell in cells {
            vector.push(cell);
        }

        Ok(
            self.create_binded_stated_heap(PengBinded::Mutable(PengStated::Initialized(
                PengValue::Vector(vector),
            ))),
        )
    }

    pub fn get_value_from_cell(&self, cell: PengCell) -> Result<PengValue, PengError> {
        match cell {
            PengCell::Nil => Ok(PengValue::Nil),
            PengCell::Int(v) => Ok(PengValue::Int(v)),
            PengCell::Uint(v) => Ok(PengValue::Uint(v)),
            PengCell::Float32(v) => Ok(PengValue::Float32(v)),
            PengCell::Float64(v) => Ok(PengValue::Float64(v)),
            PengCell::Byte(v) => Ok(PengValue::Byte(v)),
            PengCell::Bool(v) => Ok(PengValue::Bool(v)),

            PengCell::Reference(heap_ptr) => match self.get_heap(heap_ptr) {
                Some(coloured) => match &coloured.value {
                    PengBinded::Mutable(stated) | PengBinded::Immutable(stated) => match stated {
                        PengStated::Initialized(value) => Ok(value.clone()),
                        PengStated::Uninitialized => Err(PengError::Code(PengErrorCode::TestError)),
                    },
                },

                None => Err(PengError::Code(PengErrorCode::TestError)),
            },
        }
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

        let heap_ptr =
            self.create_binded_stated_heap(PengBinded::Mutable(PengStated::Uninitialized));

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

    pub fn assign_heap(&mut self, heap_ptr: PengHeapPtr, heap: PengValue) -> Result<(), PengError> {
        let coloured = match self.heap.get_mut(&heap_ptr) {
            Some(coloured) => coloured,
            None => {
                return Err(PengError::new_message("value not found".to_string()));
            }
        };

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

    pub fn create_thread(
        &mut self,
        procedure_ptr: PengHeapPtr,
        base: usize,
        params: Vec<PengBindedStatedCell>,
    ) -> PengHeapPtr {
        let thread = PengThread::new(procedure_ptr, base, params);
        let t = PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread)));
        self.create_binded_stated_heap(t)
    }

    pub fn push_thread_binded_stated_cell(
        &mut self,
        thread_ptr: PengHeapPtr,
        cell: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(thread_coloured) => match &mut thread_coloured.value {
                PengBinded::Mutable(thread_stated) => match thread_stated {
                    PengStated::Initialized(thread_value) => {
                        if let PengValue::Thread(thread) = thread_value {
                            thread.stack.push(cell);
                            return Ok(());
                        } else {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    }
                    PengStated::Uninitialized => {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                },
                PengBinded::Immutable(_) => return Err(PengError::Code(PengErrorCode::TestError)),
            },
            None => return Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_latest_binded_stated_cell(
        &mut self,
        thread_ptr: PengHeapPtr,
        offset: usize,
    ) -> Result<&PengBindedStatedCell, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(thread_coloured) => match &mut thread_coloured.value {
                PengBinded::Mutable(thread_stated) => match thread_stated {
                    PengStated::Initialized(thread_value) => {
                        if let PengValue::Thread(thread) = thread_value {
                            if thread.stack.len() <= offset {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }

                            let index = thread.stack.len() - offset - 1;

                            match thread.stack.get(index) {
                                Some(v) => return Ok(v),
                                None => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        } else {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    }

                    PengStated::Uninitialized => {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                },

                PengBinded::Immutable(_) => {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }
            },

            None => return Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_2_latests_binded_stated_cell_cloned(
        &self,
        thread_ptr: PengHeapPtr,
    ) -> Result<(PengBindedStatedCell, PengBindedStatedCell), PengError> {
        match self.get_heap(thread_ptr) {
            Some(thread_coloured) => match &thread_coloured.value {
                PengBinded::Mutable(thread_stated) => match thread_stated {
                    PengStated::Initialized(thread_value) => {
                        if let PengValue::Thread(thread) = thread_value {
                            if thread.stack.len() < 2 {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }

                            let rhs_index = thread.stack.len() - 1;
                            let lhs_index = thread.stack.len() - 2;

                            match (thread.stack.get(lhs_index), thread.stack.get(rhs_index)) {
                                (Some(lhs), Some(rhs)) => Ok((lhs.clone(), rhs.clone())),

                                _ => Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        } else {
                            Err(PengError::Code(PengErrorCode::TestError))
                        }
                    }

                    PengStated::Uninitialized => Err(PengError::Code(PengErrorCode::TestError)),
                },

                PengBinded::Immutable(_) => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_latest_n_binded_stated_cells_cloned(
        &self,
        thread_ptr: PengHeapPtr,
        n: usize,
    ) -> Result<Vec<PengBindedStatedCell>, PengError> {
        match self.get_heap(thread_ptr) {
            Some(thread_heap) => match &thread_heap.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    if thread.stack.len() < n {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }

                    Ok(thread.stack[thread.stack.len() - n..].to_vec())
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn pop_thread_stack_n_times(
        &mut self,
        thread_ptr: PengHeapPtr,
        n: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(thread_coloured) => match &mut thread_coloured.value {
                PengBinded::Mutable(thread_stated) => match thread_stated {
                    PengStated::Initialized(thread_value) => {
                        if let PengValue::Thread(thread) = thread_value {
                            for _ in 0..n {
                                match thread.stack.pop() {
                                    Some(_) => {}
                                    None => return Err(PengError::Code(PengErrorCode::TestError)),
                                }
                            }
                            return Ok(());
                        } else {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    }
                    PengStated::Uninitialized => {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                },
                PengBinded::Immutable(_) => return Err(PengError::Code(PengErrorCode::TestError)),
            },
            None => return Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_local(
        &self,
        thread_ptr: PengHeapPtr,
        local: usize,
    ) -> Result<&PengBindedStatedCell, PengError> {
        match self.get_heap(thread_ptr) {
            Some(coloured) => match coloured.value.value() {
                PengStated::Initialized(PengValue::Thread(thread)) => {
                    let base = match thread.frames.last() {
                        Some(frame) => frame.base,
                        None => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    match base.checked_add(local) {
                        Some(index) => match thread.stack.get(index) {
                            Some(cell) => Ok(cell),
                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        },

                        None => Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn set_thread_local(
        &mut self,
        thread_ptr: PengHeapPtr,
        local: usize,
        value: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    let base = match thread.frames.last() {
                        Some(frame) => frame.base,
                        None => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    match base.checked_add(local) {
                        Some(index) => match thread.stack.get_mut(index) {
                            Some(cell) => {
                                *cell = value;
                                Ok(())
                            }

                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        },

                        None => Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn set_thread_program_counter(
        &mut self,
        thread_ptr: PengHeapPtr,
        target: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    match thread.frames.last_mut() {
                        Some(frame) => {
                            if target == 0 {
                                frame.program_counter = 0;
                            } else {
                                frame.program_counter = target;
                            }

                            Ok(())
                        }
                        None => Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }
                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn push_thread_frame(
        &mut self,
        thread_ptr: PengHeapPtr,
        frame: PengFrame,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    thread.frames.push(frame);
                    Ok(())
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn pop_thread_stack_at(
        &mut self,
        thread_ptr: PengHeapPtr,
        offset: usize,
    ) -> Result<PengBindedStatedCell, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    if thread.stack.len() <= offset {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }

                    let index = thread.stack.len() - offset - 1;

                    Ok(thread.stack.remove(index))
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_stack_len(&self, thread_ptr: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_heap(thread_ptr) {
            Some(coloured) => match &coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    Ok(thread.stack.len())
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_cell_from_value(&mut self, value: PengValue) -> PengCell {
        match value {
            PengValue::Nil => PengCell::Nil,
            PengValue::Int(v) => PengCell::Int(v),
            PengValue::Uint(v) => PengCell::Uint(v),
            PengValue::Float32(v) => PengCell::Float32(v),
            PengValue::Float64(v) => PengCell::Float64(v),
            PengValue::Byte(v) => PengCell::Byte(v),
            PengValue::Bool(v) => PengCell::Bool(v),

            value => {
                let heap_ptr = self
                    .create_binded_stated_heap(PengBinded::Mutable(PengStated::Initialized(value)));

                PengCell::Reference(heap_ptr)
            }
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
                    return Err(PengError::Code(PengErrorCode::TestError));
                }

                let function_index = stack_len - args_count - 1;

                match self.get_thread_latest_binded_stated_cell(thread_ptr, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            _ => return Err(PengError::Code(PengErrorCode::TestError)),
                        };

                        match self.pop_thread_stack_at(thread_ptr, args_count) {
                            Ok(_) => {
                                match self.push_thread_frame(
                                    thread_ptr,
                                    PengFrame::new_try(function_ptr, function_index, args_count),
                                ) {
                                    Ok(()) => Ok(()),
                                    Err(e) => return Err(e),
                                }
                            }

                            Err(e) => return Err(e),
                        }
                    }

                    Err(e) => return Err(e),
                }
            }

            Err(e) => return Err(e),
        }
    }

    pub fn get_thread_latest_frame(
        &self,
        thread_ptr: PengHeapPtr,
    ) -> Result<&PengFrame, PengError> {
        match self.get_heap(thread_ptr) {
            Some(coloured) => match coloured.value.value() {
                PengStated::Initialized(PengValue::Thread(thread)) => match thread.frames.last() {
                    Some(frame) => Ok(frame),
                    None => Err(PengError::Code(PengErrorCode::TestError)),
                },

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn pop_thread_frame(&mut self, thread_ptr: PengHeapPtr) -> Result<PengFrame, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    match thread.frames.pop() {
                        Some(frame) => Ok(frame),
                        None => Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn truncate_thread_stack(
        &mut self,
        thread_ptr: PengHeapPtr,
        len: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                    thread.stack.truncate(len);
                    Ok(())
                }

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn get_thread_frames_len(&self, thread_ptr: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_heap(thread_ptr) {
            Some(coloured) => match coloured.value.value() {
                PengStated::Initialized(PengValue::Thread(thread)) => Ok(thread.frames.len()),

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    pub fn recover_thread_try_error(&mut self, thread_ptr: PengHeapPtr) -> Result<bool, PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(coloured) => match &mut coloured.value {
                PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
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

                _ => Err(PengError::Code(PengErrorCode::TestError)),
            },

            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }
}
