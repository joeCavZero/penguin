use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::binding::*;
use crate::core::cell::*;
use crate::core::colour::*;
use crate::core::error::*;
use crate::core::frame::*;
use crate::core::garbage_collector::*;
use crate::core::heap_value::*;
use crate::core::runtime::*;
use crate::core::thread::*;
use crate::core::unit::*;
use crate::core::utils::*;
use crate::core::value::*;
use crate::core::vector::*;
use crate::generator::*;
use crate::lexer::*;
use crate::parser::*;

pub struct PengEnv {
    heap: HashMap<PengHeapPtr, PengColouredValue>,
    name_pool: HashMap<PengNamePoolPtr, String>,
    active_threads: HashSet<PengHeapPtr>,
    pinned: HashSet<PengHeapPtr>,
    garbage_collector_interval: std::time::Duration,
}

impl PengEnv {
    pub fn new() -> Self {
        Self {
            heap: HashMap::new(),
            name_pool: HashMap::new(),
            pinned: HashSet::new(),
            garbage_collector_interval: std::time::Duration::from_secs(5),
            active_threads: HashSet::new(),
        }
    }

    pub fn heap(&self) -> &HashMap<PengHeapPtr, PengColouredValue> {
        &self.heap
    }

    pub fn heap_mut(&mut self) -> &mut HashMap<PengHeapPtr, PengColouredValue> {
        &mut self.heap
    }

    pub fn pinned(&self) -> &HashSet<PengHeapPtr> {
        &self.pinned
    }

    pub fn pinned_mut(&mut self) -> &mut HashSet<PengHeapPtr> {
        &mut self.pinned
    }

    pub fn active_threads(&self) -> &HashSet<PengHeapPtr> {
        &self.active_threads
    }

    pub fn load_script_from_file(&mut self, path: &str) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_script_from_file_using(path, &using_unit)
    }

    pub fn load_script_from_file_using(
        &mut self,
        path: &str,
        using_unit: &PengUnit,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_file(path.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let unit = match generate_ast_using(self, &ast, using_unit) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(unit)
    }

    pub fn load_script_from_source(&mut self, source: &str) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_script_from_source_using(source, &using_unit)
    }

    pub fn load_script_from_source_using(
        &mut self,
        source: &str,
        using_unit: &PengUnit,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_source(source.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_script(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let unit = match generate_ast_using(self, &ast, using_unit) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(unit)
    }

    pub fn load_program_from_file(&mut self, path: &str) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_program_from_file_using(path, &using_unit)
    }

    pub fn load_program_from_file_using(
        &mut self,
        path: &str,
        using_unit: &PengUnit,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_file(path.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_program(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let unit = match generate_ast_using(self, &ast, using_unit) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(unit)
    }

    pub fn load_program_from_source(&mut self, source: &str) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_program_from_source_using(source, &using_unit)
    }

    pub fn load_program_from_source_using(
        &mut self,
        source: &str,
        using_unit: &PengUnit,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_source(source.to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let ast = match parse_program(tokens) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let unit = match generate_ast_using(self, &ast, using_unit) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(unit)
    }

    pub fn run(&mut self, init: PengHeapPtr) -> Result<PengBindedCell, PengError> {
        let main_thread = self.create_thread(init, 0, Vec::new(), PengThreadState::Running);

        self.run_scheduler(main_thread)
    }

    pub fn run_function(
        &mut self,
        unit: &PengUnit,
        name: &str,
        args: Vec<PengBindedCell>,
    ) -> Result<PengBindedCell, PengError> {
        let name_ptr = self.ensure_pooled_name_ptr(name.to_string());

        let function_ptr = match unit.get_global(name_ptr) {
            Some(value) => *value.value(),
            None => return Err(PengError::NameNotFound(name_ptr)),
        };

        match self.get_heap(function_ptr) {
            Some(PengValue::Box(PengBox::Function(_))) => {}
            Some(_) => return Err(PengError::ExpectedFunction),
            None => return Err(PengError::HeapValueNotFound(function_ptr)),
        }

        let main_thread = self.create_thread(function_ptr, 0, args, PengThreadState::Running);

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

                    Err(PengError::ThreadNotFound(_)) => {
                        self.active_threads.remove(&thread_ptr);
                        continue;
                    }

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

                                self.active_threads.remove(&thread_ptr);

                                if thread_ptr == main_thread {
                                    return Ok(result);
                                }
                            }

                            Ok(None) => {}

                            Err(e) => {
                                {
                                    let thread = match self.get_thread_mut(thread_ptr) {
                                        Ok(v) => v,
                                        Err(e) => return Err(e),
                                    };

                                    thread.result = PengThreadResult::Failed(Box::new(e.clone()));
                                    thread.state = PengThreadState::Failed;
                                }

                                self.active_threads.remove(&thread_ptr);

                                if thread_ptr == main_thread {
                                    return Err(e);
                                }
                            }
                        }
                    }

                    PengThreadState::Finished
                    | PengThreadState::Failed
                    | PengThreadState::Cancelled => {
                        self.active_threads.remove(&thread_ptr);
                    }

                    PengThreadState::Paused | PengThreadState::Waiting => {}
                }
            }

            if !any_running {
                return Err(PengError::InvalidState("no running threads".to_string()));
            }
        }
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

    pub fn get_pooled_name(&self, name: PengNamePoolPtr) -> Option<&String> {
        self.name_pool.get(&name)
    }

    pub fn create_heap_value(&mut self, value: PengValue) -> PengHeapPtr {
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

        Ok(self.create_heap_value(PengValue::Box(PengBox::Vector(vector))))
    }

    pub fn get_coloured_heap(&self, value: PengHeapPtr) -> Option<&PengColouredValue> {
        self.heap.get(&value)
    }

    pub fn get_coloured_heap_mut(&mut self, value: PengHeapPtr) -> Option<&mut PengColouredValue> {
        self.heap.get_mut(&value)
    }

    pub fn get_heap(&self, value: PengHeapPtr) -> Option<&PengValue> {
        match self.heap.get(&value) {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    pub fn get_heap_mut(&mut self, value: PengHeapPtr) -> Option<&mut PengValue> {
        match self.heap.get_mut(&value) {
            Some(v) => Some(&mut v.value),
            None => None,
        }
    }

    pub fn assign_heap(&mut self, heap: PengHeapPtr, value: PengValue) -> Result<(), PengError> {
        match self.get_heap_mut(heap) {
            Some(heap_value) => {
                *heap_value = value;
                return Ok(());
            }
            None => {
                return Err(PengError::HeapValueNotFound(heap));
            }
        };
    }

    pub fn create_thread(
        &mut self,
        procedure: PengHeapPtr,
        base: usize,
        params: Vec<PengBindedCell>,
        state: PengThreadState,
    ) -> PengHeapPtr {
        let t = PengValue::Box(PengBox::Thread(PengThread::new(
            procedure, base, params, state,
        )));
        let tptr = self.create_heap_value(t);
        self.active_threads.insert(tptr);
        tptr
    }

    pub fn push_thread_binded_stated_cell(
        &mut self,
        thread: PengHeapPtr,
        cell: PengBindedCell,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread) {
            Ok(thread) => {
                thread.stack.push(cell);
                Ok(())
            }
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn get_thread_latest_binded_stated_cell(
        &self,
        thread: PengHeapPtr,
        offset: usize,
    ) -> Result<&PengBindedCell, PengError> {
        match self.get_thread(thread) {
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

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn get_thread_2_latests_binded_stated_cell_cloned(
        &self,
        thread: PengHeapPtr,
    ) -> Result<(PengBindedCell, PengBindedCell), PengError> {
        match self.get_heap(thread) {
            Some(h) => {
                if let PengValue::Box(PengBox::Thread(thread)) = h {
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
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn get_thread_latest_n_binded_stated_cells_cloned(
        &self,
        thread: PengHeapPtr,
        n: usize,
    ) -> Result<Vec<PengBindedCell>, PengError> {
        match self.get_thread(thread) {
            Ok(thread) => {
                if thread.stack.len() < n {
                    return Err(PengError::StackUnderflow);
                }

                Ok(thread.stack[thread.stack.len() - n..].to_vec())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn pop_thread_stack_n_times(
        &mut self,
        thread: PengHeapPtr,
        n: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread) {
            Ok(thread) => {
                for _ in 0..n {
                    match thread.stack.pop() {
                        Some(_) => {}
                        None => return Err(PengError::StackUnderflow),
                    }
                }

                Ok(())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn get_thread_local(
        &self,
        thread: PengHeapPtr,
        local: usize,
    ) -> Result<&PengBindedCell, PengError> {
        match self.get_heap(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
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
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn set_thread_local(
        &mut self,
        thread: PengHeapPtr,
        local: usize,
        value: PengBindedCell,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
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
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn set_thread_program_counter(
        &mut self,
        thread: PengHeapPtr,
        target: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread) {
            Ok(thread) => match thread.frames.last_mut() {
                Some(frame) => {
                    frame.program_counter = target;
                    Ok(())
                }
                None => Err(PengError::FrameNotFound),
            },
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn push_thread_frame(
        &mut self,
        thread: PengHeapPtr,
        frame: PengFrame,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread) {
            Ok(thread) => {
                thread.frames.push(frame);
                Ok(())
            }

            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn pop_thread_stack_at(
        &mut self,
        thread: PengHeapPtr,
        offset: usize,
    ) -> Result<PengBindedCell, PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                if thread.stack.len() <= offset {
                    return Err(PengError::StackUnderflow);
                }

                let index = thread.stack.len() - offset - 1;

                Ok(thread.stack.remove(index))
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn get_thread_stack_len(&self, thread: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_thread(thread) {
            Ok(thread) => Ok(thread.stack.len()),
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn execute_try_function_call(
        &mut self,
        thread: PengHeapPtr,
        args_count: usize,
    ) -> Result<(), PengError> {
        match self.get_thread_stack_len(thread) {
            Ok(stack_len) => {
                if stack_len < args_count + 1 {
                    return Err(PengError::TooFewArguments {
                        expected: args_count + 1,
                        found: stack_len,
                    });
                }

                let function_index = stack_len - args_count - 1;

                match self.get_thread_latest_binded_stated_cell(thread, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
                        };

                        match self.pop_thread_stack_at(thread, args_count) {
                            Ok(_) => {
                                match self.push_thread_frame(
                                    thread,
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

            Err(e) => return Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }

    pub fn get_thread_latest_frame(&self, thread: PengHeapPtr) -> Result<&PengFrame, PengError> {
        match self.get_heap(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => match thread.frames.last() {
                Some(frame) => Ok(frame),
                None => Err(PengError::FrameNotFound),
            },
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn pop_thread_frame(&mut self, thread: PengHeapPtr) -> Result<PengFrame, PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => match thread.frames.pop() {
                Some(frame) => Ok(frame),
                None => Err(PengError::FrameNotFound),
            },
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn truncate_thread_stack(
        &mut self,
        thread: PengHeapPtr,
        len: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                thread.stack.truncate(len);
                Ok(())
            }
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn get_thread_frames_len(&self, thread: PengHeapPtr) -> Result<usize, PengError> {
        match self.get_heap(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => Ok(thread.frames.len()),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn recover_thread_try_error(&mut self, thread: PengHeapPtr) -> Result<bool, PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
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
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    fn get_thread(&self, thread: PengHeapPtr) -> Result<&PengThread, PengError> {
        match self.get_heap(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => Ok(thread),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    fn get_thread_mut(&mut self, thread: PengHeapPtr) -> Result<&mut PengThread, PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => Ok(thread),
            Some(_) => Err(PengError::ExpectedThread),
            None => Err(PengError::ThreadNotFound(thread)),
        }
    }

    pub fn get_value_from_cell(&self, cell: PengCell) -> Result<PengValue, PengError> {
        match cell {
            PengCell::Reference(ptr) => match self.get_heap(ptr) {
                Some(value) => Ok(value.clone()),
                None => Err(PengError::HeapValueNotFound(ptr)),
            },

            _ => Ok(PengValue::Cell(cell)),
        }
    }

    pub fn get_box_from_cell(&self, cell: PengCell) -> Result<PengBox, PengError> {
        match cell {
            PengCell::Reference(ptr) => match self.get_heap(ptr) {
                Some(PengValue::Box(value)) => Ok(value.clone()),
                _ => Err(PengError::HeapValueNotFound(ptr)),
            },

            _ => Err(PengError::ExpectedReference),
        }
    }

    pub fn get_cell_from_value(&mut self, value: PengValue) -> Result<PengCell, PengError> {
        match value {
            PengValue::Cell(cell) => Ok(cell),
            PengValue::Box(value) => {
                let ptr = self.create_heap_value(PengValue::Box(value));
                Ok(PengCell::Reference(ptr))
            }
        }
    }

    pub fn set_thread_state(
        &mut self,
        thread: PengHeapPtr,
        state: PengThreadState,
    ) -> Result<(), PengError> {
        match self.get_thread_mut(thread) {
            Ok(thread) => {
                thread.state = state;
                Ok(())
            }
            Err(e) => Err(e.push(PengError::ThreadNotFound(thread))),
        }
    }
}
