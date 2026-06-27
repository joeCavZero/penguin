use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::binding::*;
use crate::core::cell::*;
use crate::core::colour::*;
use crate::core::error::*;
use crate::core::frame::*;
use crate::core::function::*;
use crate::core::garbage_collector::*;
use crate::core::heap_value::*;
use crate::core::operation::*;
use crate::core::runtime::*;
use crate::core::thread::*;
use crate::core::utils::*;
use crate::core::value::*;
use crate::core::vector::*;
use crate::generator::*;
use crate::lexer::*;
use crate::parser::*;

pub struct PengEnv {
    globals: HashMap<PengNamePoolPtr, PengBindedCell>,
    pub heap: HashMap<PengHeapPtr, PengColouredHeapValue>,
    pub name_pool: HashMap<PengNamePoolPtr, String>,
    active_threads: HashSet<PengHeapPtr>,
    garbage_collector_interval: std::time::Duration,
}

impl PengEnv {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            heap: HashMap::new(),
            name_pool: HashMap::new(),
            garbage_collector_interval: std::time::Duration::from_secs(5),
            active_threads: HashSet::new(),
        }
    }

    pub fn load_script_from_file(&mut self, path: &str) -> Result<PengHeapPtr, PengError> {
        let tokens = match lex_file(path.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let init_ptr = match generate_ast(self, &ast) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(init_ptr)
    }

    pub fn load_script_from_source(&mut self, source: &str) -> Result<PengHeapPtr, PengError> {
        let tokens = match lex_source(source.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let init_ptr = match generate_ast(self, &ast) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(init_ptr)
    }

    pub fn load_program_from_file(&mut self, path: &str) -> Result<PengHeapPtr, PengError> {
        let tokens = match lex_file(path.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let init_ptr = match generate_ast(self, &ast) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(init_ptr)
    }

    pub fn load_program_from_source(&mut self, source: &str) -> Result<PengHeapPtr, PengError> {
        let tokens = match lex_source(source.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let init_ptr = match generate_ast(self, &ast) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(init_ptr)
    }

    pub fn run(&mut self, init_ptr: PengHeapPtr) -> Result<PengBindedCell, PengError> {
        let main_thread = self.create_thread(init_ptr, 0, Vec::new(), PengThreadState::Running);

        self.run_scheduler(main_thread)
    }

    pub fn run_scheduler(&mut self, main_thread: PengHeapPtr) -> Result<PengBindedCell, PengError> {
        let gc_interval = self.garbage_collector_interval;
        let mut last_gc = std::time::Instant::now();

        loop {
            if last_gc.elapsed() >= gc_interval {
                self.mark_and_sweep();
                last_gc = std::time::Instant::now();
            }

            let threads: Vec<PengHeapPtr> = self.active_threads.iter().cloned().collect();

            let mut any_running = false;

            for thread_ptr in threads {
                let state = match self.get_thread(thread_ptr) {
                    Ok(v) => v.state.clone(),
                    Err(e) => return Err(e),
                };

                match state {
                    PengThreadState::Running => {
                        any_running = true;

                        match step_thread(self, thread_ptr) {
                            Ok(Some(result)) => {
                                {
                                    let thread = match self.get_thread_mut(thread_ptr) {
                                        Ok(v) => v,
                                        Err(e) => return Err(e),
                                    };
                                    thread.result = PengThreadResult::Returned(result.clone());
                                    thread.state = PengThreadState::Finished;
                                }

                                if thread_ptr == main_thread {
                                    return Ok(result);
                                }
                            }

                            Ok(None) => {}

                            Err(e) => {
                                let thread = match self.get_thread_mut(thread_ptr) {
                                    Ok(v) => v,
                                    Err(e) => return Err(e),
                                };
                                thread.result = PengThreadResult::Failed(Box::new(e.clone()));
                                thread.state = PengThreadState::Failed;
                            }
                        }
                    }

                    PengThreadState::Finished
                    | PengThreadState::Failed
                    | PengThreadState::Paused
                    | PengThreadState::Waiting
                    | PengThreadState::Cancelled => {}
                }
            }

            if !any_running {
                return Err(PengError::InvalidState("no running threads".to_string()));
            }
        }
    }

    pub fn register_native_function<F>(&mut self, name: &str, func: F) -> Result<(), PengError>
    where
        F: FnMut(Vec<PengBindedCell>, &mut PengEnv) -> Result<PengBindedCell, PengError> + 'static,
    {
        let ptr = self.create_heap_value(PengHeapValue::Function(PengFunction::new_native(func)));

        let value = PengBindedCell::Immutable(PengCell::Reference(ptr));

        self.set_global(name.to_string(), value)
    }

    pub fn register_native_operation<F>(&mut self, name: &str, op: F) -> Result<(), PengError>
    where
        F: FnMut(
                (PengBindedCell, PengBindedCell),
                &mut PengEnv,
            ) -> Result<PengBindedCell, PengError>
            + 'static,
    {
        let ptr = self.create_heap_value(PengHeapValue::Operation(PengOperation::new_native(op)));

        let value = PengBindedCell::Immutable(PengCell::Reference(ptr));

        self.set_global(name.to_string(), value)
    }

    fn next_name_ptr(&self) -> PengNamePoolPtr {
        self.name_pool
            .keys()
            .max()
            .map(|v| *v + PengNamePoolPtr(1).into())
            .unwrap_or(PengNamePoolPtr(0))
    }

    fn next_heap_ptr(&self) -> PengHeapPtr {
        self.heap
            .keys()
            .max()
            .map(|v| *v + PengHeapPtr(1).into())
            .unwrap_or(PengHeapPtr(0))
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
        cells: impl IntoIterator<Item = PengBindedCell>,
    ) -> Result<PengHeapPtr, PengError> {
        let mut vector = PengVector::new_empty();

        for cell in cells {
            vector.push(cell);
        }

        Ok(self.create_heap_value(PengHeapValue::Vector(vector)))
    }

    pub fn has_global(&self, name_ptr: PengNamePoolPtr) -> bool {
        self.globals.contains_key(&name_ptr)
    }

    pub fn create_global(
        &mut self,
        name_ptr: PengNamePoolPtr,
        value: PengBindedCell,
    ) -> Result<(), PengError> {
        if self.globals.contains_key(&name_ptr) {
            return Err(PengError::SyntaxError(
                "duplicated global declaration".to_string(),
            ));
        }

        self.globals.insert(name_ptr, value);
        Ok(())
    }

    pub fn set_global(&mut self, name: String, cell: PengBindedCell) -> Result<(), PengError> {
        let name_ptr = self.ensure_pooled_name_ptr(name);

        match self.globals.get_mut(&name_ptr) {
            Some(current_cell) => match current_cell {
                PengBinded::Mutable(current) => {
                    *current = cell.value().clone();
                    Ok(())
                }

                PengBinded::Immutable(_) => {
                    *current_cell = cell;
                    Ok(())
                }
            },

            None => {
                self.globals.insert(name_ptr, cell);
                Ok(())
            }
        }
    }

    pub fn get_global_by_name_str(&self, name: &str) -> Option<&PengBindedCell> {
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

    pub fn get_coloured_heap(&self, value_ptr: PengHeapPtr) -> Option<&PengColouredHeapValue> {
        match self.heap.get(&value_ptr) {
            Some(v) => Some(&v),
            None => None,
        }
    }

    pub fn get_coloured_heap_mut(
        &mut self,
        value_ptr: PengHeapPtr,
    ) -> Option<&mut PengColouredHeapValue> {
        match self.heap.get_mut(&value_ptr) {
            Some(v) => Some(v),
            None => None,
        }
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
        params: Vec<PengBindedCell>,
        state: PengThreadState,
    ) -> PengHeapPtr {
        let t = PengHeapValue::Thread(PengThread::new(procedure_ptr, base, params, state));
        let tptr = self.create_heap_value(t);
        self.active_threads.insert(tptr);
        tptr
    }

    pub fn push_thread_binded_stated_cell(
        &mut self,
        thread_ptr: PengHeapPtr,
        cell: PengBindedCell,
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
    ) -> Result<&PengBindedCell, PengError> {
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
    ) -> Result<(PengBindedCell, PengBindedCell), PengError> {
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
    ) -> Result<Vec<PengBindedCell>, PengError> {
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
    ) -> Result<&PengBindedCell, PengError> {
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
        value: PengBindedCell,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                let base = match thread.frames.last() {
                    Some(frame) => frame.base,
                    None => return Err(PengError::FrameNotFound),
                };

                match base.checked_add(local) {
                    Some(index) => match thread.stack.get_mut(index) {
                        Some(cell) => match cell {
                            PengBinded::Mutable(_) => {
                                *cell = value;
                                Ok(())
                            }

                            PengBinded::Immutable(_) => {
                                *cell = value;
                                Ok(())
                            }
                        },
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
    ) -> Result<PengBindedCell, PengError> {
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
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
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

                        thread.stack.push(PengBinded::Mutable(PengCell::Nil));

                        thread
                            .stack
                            .push(PengBinded::Mutable(PengCell::Bool(false)));

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

    pub fn set_thread_state(
        &mut self,
        thread_ptr: PengHeapPtr,
        state: PengThreadState,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread_ptr) {
            Ok(thread) => {
                thread.state = state;
                Ok(())
            }
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread_ptr))),
        }
    }
}
