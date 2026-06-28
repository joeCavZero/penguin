use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::binding::*;
use crate::core::r#box::*;
use crate::core::cell::*;
use crate::core::colour::*;
use crate::core::error::*;
use crate::core::frame::*;
use crate::core::function::*;
use crate::core::garbage_collector::*;
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

    pub fn active_threads_mut(&mut self) -> &mut HashSet<PengHeapPtr> {
        &mut self.active_threads
    }

    pub fn activate_thread(&mut self, thread: PengHeapPtr) {
        self.active_threads.insert(thread);
    }

    pub fn load_script_from_file(
        &mut self,
        path: &str,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_script_from_file_using(path, &using_unit, position_id)
    }

    pub fn load_script_from_file_using(
        &mut self,
        path: &str,
        using_unit: &PengUnit,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_file(path.to_string(), position_id) {
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

    pub fn load_script_from_source(
        &mut self,
        position_id: usize,
        source: &str,
    ) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_script_from_source_using(source, &using_unit, position_id)
    }

    pub fn load_script_from_source_using(
        &mut self,
        source: &str,
        using_unit: &PengUnit,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_source(source.to_string(), position_id) {
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

    pub fn load_program_from_file(
        &mut self,
        path: &str,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_program_from_file_using(path, &using_unit, position_id)
    }

    pub fn load_program_from_file_using(
        &mut self,
        path: &str,
        using_unit: &PengUnit,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_file(path.to_string(), position_id) {
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

    pub fn load_program_from_source(
        &mut self,
        source: &str,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let using_unit = PengUnit::library();

        self.load_program_from_source_using(source, &using_unit, position_id)
    }

    pub fn load_program_from_source_using(
        &mut self,
        source: &str,
        using_unit: &PengUnit,
        position_id: usize,
    ) -> Result<PengUnit, PengError> {
        let tokens = match lex_source(source.to_string(), position_id) {
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

    pub fn run(&mut self, init: PengHeapPtr, unit: &PengUnit) -> Result<PengBindedCell, PengError> {
        let main_thread = self.create_thread(init, 0, Vec::new(), PengThreadState::Running);

        self.run_scheduler(main_thread, unit)
    }

    pub fn run_global_function(
        &mut self,
        name: &str,
        unit: &PengUnit,
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

        self.run_scheduler(main_thread, unit)
    }

    pub fn run_scheduler(
        &mut self,
        main_thread: PengHeapPtr,
        unit: &PengUnit,
    ) -> Result<PengBindedCell, PengError> {
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

                        match step_thread(self, thread_ptr, unit) {
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

                            Ok(None) => {
                                let should_yield = {
                                    let thread = match self.get_thread_mut(thread_ptr) {
                                        Ok(v) => v,
                                        Err(e) => return Err(e),
                                    };

                                    if thread.quantum == 0 {
                                        thread.quantum = 1;
                                        true
                                    } else {
                                        false
                                    }
                                };

                                if should_yield {
                                    continue;
                                }
                            }

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

    pub fn reserve_thread_local(
        &mut self,
        thread: PengHeapPtr,
        local: usize,
    ) -> Result<(), PengError> {
        match self.get_heap_mut(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
                let frame = match thread.frames.last_mut() {
                    Some(frame) => frame,
                    None => return Err(PengError::FrameNotFound),
                };

                if frame.reserved_locals.contains(&local) {
                    return Ok(());
                }

                let index = match frame.base.checked_add(local) {
                    Some(index) => index,
                    None => return Err(PengError::InvalidFrame),
                };

                if index > thread.stack.len() {
                    return Err(PengError::LocalNotFound(local));
                }

                thread
                    .stack
                    .insert(index, PengBinded::Mutable(PengCell::Nil));
                frame.reserved_locals.push(local);

                Ok(())
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

    fn push_call_frame(
        &mut self,
        thread: PengHeapPtr,
        function_ptr: PengHeapPtr,
        base: usize,
        params_count: usize,
        is_try: bool,
    ) -> Result<(), PengError> {
        let frame = if is_try {
            PengFrame::new_try(function_ptr, base, params_count)
        } else {
            PengFrame::new(function_ptr, base, params_count)
        };

        self.push_thread_frame(thread, frame)
    }

    pub fn execute_function_call(
        &mut self,
        thread: PengHeapPtr,
        args_count: usize,
        is_try: bool,
    ) -> Result<(), PengError> {
        let stack_len = match self.get_thread_stack_len(thread) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        // Isso não é "argumento faltando" da linguagem.
        // Isso é stack malformada: nem existe callee suficiente na stack.
        if stack_len < args_count + 1 {
            return Err(PengError::StackUnderflow);
        }

        let function_index = stack_len - args_count - 1;

        let function_ptr = {
            let function_cell = match self.get_thread_latest_binded_stated_cell(thread, args_count)
            {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            match function_cell.value() {
                PengCell::Reference(ptr) => *ptr,
                _ => return Err(PengError::ExpectedReference),
            }
        };

        let function = match self.get_heap(function_ptr) {
            Some(PengValue::Box(PengBox::Function(function))) => function.clone(),
            Some(_) => return Err(PengError::ExpectedFunction),
            None => return Err(PengError::HeapValueNotFound(function_ptr)),
        };

        match function {
            PengFunction::Native(_) => {
                // Native ainda não tem metadado de quantidade esperada.
                // Então mantém comportamento dinâmico: passa tudo que foi enviado.
                match self.pop_thread_stack_at(thread, args_count) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                };

                match self.push_call_frame(thread, function_ptr, function_index, args_count, is_try)
                {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                };

                Ok(())
            }

            PengFunction::Bytecode(func_btc) => {
                let args =
                    match self.get_thread_latest_n_binded_stated_cells_cloned(thread, args_count) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                // Remove função + todos os argumentos reais.
                match self.pop_thread_stack_n_times(thread, args_count + 1) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                };

                match func_btc.params {
                    PengBytecodeFunctionParams::Fixed(expected_count) => {
                        // Usa só os argumentos necessários.
                        // Se faltar, completa com nil.
                        for i in 0..expected_count {
                            let arg = match args.get(i) {
                                Some(arg) => arg.clone(),
                                None => PengBinded::Mutable(PengCell::Nil),
                            };

                            match self.push_thread_binded_stated_cell(thread, arg) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            };
                        }

                        match self.push_call_frame(
                            thread,
                            function_ptr,
                            function_index,
                            expected_count,
                            is_try,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };

                        Ok(())
                    }

                    PengBytecodeFunctionParams::Variadic(fixed_count) => {
                        // Primeiro empilha os argumentos fixos.
                        // Se faltar algum fixo, completa com nil.
                        for i in 0..fixed_count {
                            let arg = match args.get(i) {
                                Some(arg) => arg.clone(),
                                None => PengBinded::Mutable(PengCell::Nil),
                            };

                            match self.push_thread_binded_stated_cell(thread, arg) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            };
                        }

                        // O resto vira vetor.
                        // Se não tiver resto, vira [].
                        let rest = if args.len() > fixed_count {
                            args[fixed_count..].to_vec()
                        } else {
                            Vec::new()
                        };

                        let vector_ptr = self.create_heap_value(PengValue::Box(PengBox::Vector(
                            PengVector::new(rest),
                        )));

                        match self.push_thread_binded_stated_cell(
                            thread,
                            PengBinded::Mutable(PengCell::Reference(vector_ptr)),
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };

                        match self.push_call_frame(
                            thread,
                            function_ptr,
                            function_index,
                            fixed_count + 1,
                            is_try,
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        };

                        Ok(())
                    }
                }
            }
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
