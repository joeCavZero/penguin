use crate::core::*;

pub fn step_thread(
    env: &mut PengEnv,
    thread_ptr: PengHeapPtr,
) -> Result<Option<PengBindedStatedCell>, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, frame_params_count) =
        match env.get_heap(thread_ptr) {
            Some(PengHeapValue::Thread(thread)) => {
                if let Some(last_frame) = thread.frames.last() {
                    (
                        last_frame.program_counter,
                        last_frame.base,
                        last_frame.procedure,
                        last_frame.params_count,
                    )
                } else {
                    return Ok(None);
                }
            }

            Some(_) => {
                return Err(PengError::ExpectedThread);
            }

            None => {
                return Err(PengError::ThreadNotFound(thread_ptr));
            }
        };

    let (ntv_opt, should_end_frame, instruction, constant): (
        Option<PengNativeFunction>,
        bool,
        PengInstruction,
        Option<PengValue>,
    ) = match env.get_heap(frame_function_ptr) {
        Some(PengHeapValue::Function(func)) => match func {
            PengFunction::Bytecode(func_btc) => {
                let instr = match func_btc.bytecode.get(frame_program_counter) {
                    Some(i) => i.clone(),
                    None => {
                        return Ok(None);
                    }
                };

                let constant = match instr {
                    PengInstruction::PushConst(const_index) => {
                        match func_btc.consts.get(const_index) {
                            Some(v) => Some(v.clone()),
                            None => {
                                return Err(PengError::IndexOutOfBounds {
                                    index: const_index,
                                    len: func_btc.consts.len(),
                                });
                            }
                        }
                    }

                    _ => None,
                };

                (None, false, instr, constant)
            }

            PengFunction::Native(func_ntv) => {
                (Some(func_ntv.clone()), false, PengInstruction::Add, None)
            }
        },
        Some(PengHeapValue::Operation(operation)) => match operation {
            PengOperation::Bytecode(operation_btc) => {
                let instr = match operation_btc.bytecode.get(frame_program_counter) {
                    Some(i) => i.clone(),
                    None => {
                        return Ok(None);
                    }
                };

                let constant = match instr {
                    PengInstruction::PushConst(const_index) => {
                        match operation_btc.consts.get(const_index) {
                            Some(v) => Some(v.clone()),
                            None => {
                                return Err(PengError::IndexOutOfBounds {
                                    index: const_index,
                                    len: operation_btc.consts.len(),
                                });
                            }
                        }
                    }

                    _ => None,
                };

                (None, false, instr, constant)
            }

            PengOperation::Native(_) => {
                return Err(PengError::NotImplemented(
                    "native operations are not executable yet".to_string(),
                ));
            }
        },
        Some(_) => {
            return Err(PengError::ExpectedFunction);
        }
        None => {
            return Err(PengError::HeapValueNotFound(frame_function_ptr));
        }
    };

    match ntv_opt {
        Some(ntv_func) => {
            let args = match env
                .get_thread_latest_n_binded_stated_cells_cloned(thread_ptr, frame_params_count)
            {
                Ok(args) => {
                    let mut fargs: Vec<PengBindedCell> = Vec::new();
                    for a in args {
                        match a {
                            PengBinded::Immutable(s) => match s {
                                PengStated::Initialized(c) => fargs.push(PengBinded::Immutable(c)),
                                PengStated::Uninitialized => {
                                    return Err(PengError::CannotReadUninitialized);
                                }
                            },
                            PengBinded::Mutable(s) => match s {
                                PengStated::Initialized(c) => fargs.push(PengBinded::Mutable(c)),
                                PengStated::Uninitialized => {
                                    return Err(PengError::CannotReadUninitialized);
                                }
                            },
                        }
                    }
                    fargs
                }
                Err(e) => {
                    return Err(e.push(PengError::ExpectedFunction));
                }
            };

            let ret = match ntv_func.call(args, env) {
                Ok(ret) => ret,
                Err(e) => {
                    return Err(e.push(PengError::CannotCallValue(
                        "native function call failed".to_string(),
                    )));
                }
            };

            let frame = match env.pop_thread_frame(thread_ptr) {
                Ok(frame) => frame,
                Err(e) => {
                    return Err(e.push(PengError::ExpectedFunction));
                }
            };

            match env.truncate_thread_stack(thread_ptr, frame.base) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed to truncate stack after native function call".to_string(),
                    )));
                }
            }

            match env.get_thread_frames_len(thread_ptr) {
                Ok(frames_len) => {
                    let r = match ret {
                        PengBinded::Immutable(c) => {
                            PengBinded::Immutable(PengStated::Initialized(c))
                        }
                        PengBinded::Mutable(c) => PengBinded::Mutable(PengStated::Initialized(c)),
                    };
                    if frames_len == 0 {
                        return Ok(Some(r));
                    }

                    match env.push_thread_binded_stated_cell(thread_ptr, r) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed to push native function return value".to_string(),
                            )));
                        }
                    }

                    if frame.is_try {
                        match env.push_thread_binded_stated_cell(
                            thread_ptr,
                            PengBinded::Mutable(PengStated::Initialized(PengCell::Bool(true))),
                        ) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidState(
                                    "failed to push try success flag after native function call"
                                        .to_string(),
                                )));
                            }
                        }
                    }

                    return Ok(None);
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed to get frame length after native function call".to_string(),
                    )));
                }
            }
        }
        None => {
            if should_end_frame {
                return Ok(None);
            }

            match env.get_heap_mut(thread_ptr) {
                Some(PengHeapValue::Thread(thread)) => {
                    if let Some(last_frame) = thread.frames.last_mut() {
                        last_frame.program_counter = match last_frame.program_counter.checked_add(1)
                        {
                            Some(r) => r,
                            None => return Err(PengError::ArithmeticOverflow),
                        };
                    } else {
                        return Ok(None);
                    }
                }
                Some(_) => return Err(PengError::ExpectedThread),
                None => return Err(PengError::ThreadNotFound(thread_ptr)),
            }

            match execute_instruction(instruction.clone(), constant, thread_ptr, frame_base, env) {
                Ok(res) => match res {
                    Some(ret) => {
                        let frame = match env.pop_thread_frame(thread_ptr) {
                            Ok(frame) => frame,
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        };

                        match env.truncate_thread_stack(thread_ptr, frame.base) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }

                        match env.get_thread_frames_len(thread_ptr) {
                            Ok(frames_len) => {
                                if frames_len == 0 {
                                    return Ok(Some(ret));
                                }

                                match env.push_thread_binded_stated_cell(thread_ptr, ret) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }

                                if frame.is_try {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(true),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }
                            }

                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    None => {}
                },

                Err(e) => match env.recover_thread_try_error(thread_ptr) {
                    Ok(recovered) => {
                        if !recovered {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },
            };
        }
    }

    Ok(None)
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: Option<PengValue>,
    thread_ptr: PengHeapPtr,
    _frame_base: usize,
    env: &mut PengEnv,
) -> Result<Option<PengBindedStatedCell>, PengError> {
    match instruction {
        PengInstruction::PushConst(_) => {
            let constant = match constant {
                Some(value) => value,
                None => return Err(PengError::InstructionExpectedConstant),
            };

            match constant {
                PengValue::Cell(cell) => {
                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(cell)),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                PengValue::Heap(value) => {
                    let ptr = env.create_heap_value(value);
                    let cell = PengCell::Reference(ptr);

                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(cell)),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
            }
        }

        PengInstruction::PushLocal(local) => match env.get_thread_local(thread_ptr, local).cloned()
        {
            Ok(cell) => match env.push_thread_binded_stated_cell(thread_ptr, cell.clone()) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            },
            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::StoreLocal(local) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(lst) => match env.set_thread_local(thread_ptr, local, lst.clone()) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::PushHeap(ptr) => {
            let value = match env.get_heap(ptr) {
                Some(value) => value.clone(),
                None => return Err(PengError::HeapValueNotFound(ptr)),
            };

            match env.get_cell_from_value(PengValue::Heap(value.clone())) {
                Ok(c) => {
                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(c)),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(_) => {
                    let ptr = env.create_heap_value(value);
                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(ptr))),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
            }
        }

        PengInstruction::PushHeapRef(ptr) => {
            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(ptr))),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::StoreHeap => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(value_cell) => match env
                    .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                    .cloned()
                {
                    Ok(ref_cell) => {
                        let ptr: PengHeapPtr = match ref_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        let value = match value_cell.value() {
                            PengStated::Initialized(cell) => {
                                match env.get_heap_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => match env.assign_heap(ptr, value) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            },
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateEmptyObject => {
            let heap_ptr = env.create_heap_value(PengHeapValue::Object(PengObject::new_empty()));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateEmptyModule => {
            let heap_ptr = env.create_heap_value(PengHeapValue::Module(PengModule::new_empty()));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateVector(size) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread_ptr, size) {
                Ok(cells) => match env.pop_thread_stack_n_times(thread_ptr, size) {
                    Ok(()) => {
                        let heap_ptr =
                            env.create_heap_value(PengHeapValue::Vector(PengVector::new(cells)));

                        match env.push_thread_binded_stated_cell(
                            thread_ptr,
                            PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                heap_ptr,
                            ))),
                        ) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateSuperType(types_count) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread_ptr, types_count) {
                Ok(cells) => {
                    let mut custom_types = Vec::new();

                    for cell in cells {
                        match cell.value() {
                            PengStated::Initialized(cell) => {
                                match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => match value {
                                        PengValue::Heap(PengHeapValue::Type(PengType::Custom(
                                            custom_type,
                                        ))) => {
                                            custom_types.push(custom_type);
                                        }

                                        _ => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    },

                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }

                            PengStated::Uninitialized => {
                                return Err(PengError::InvalidInstruction(instruction));
                            }
                        }
                    }

                    match env.pop_thread_stack_n_times(thread_ptr, types_count) {
                        Ok(()) => {
                            let heap_ptr = env.create_heap_value(PengHeapValue::Type(
                                PengType::Custom(PengCustomType::new_super_type(custom_types)),
                            ));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateTypedObject => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => {
                    let custom_type = match cell.value() {
                        PengStated::Initialized(cell) => {
                            match env.get_value_from_cell(cell.clone()) {
                                Ok(PengValue::Heap(PengHeapValue::Type(PengType::Custom(
                                    custom_type,
                                )))) => custom_type,

                                Ok(_) => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        PengStated::Uninitialized => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            let heap_ptr = env.create_heap_value(PengHeapValue::Object(
                                PengObject::new(custom_type.fields),
                            ));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Duplicate => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => match env.push_thread_binded_stated_cell(thread_ptr, cell.clone()) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Pop => match env.pop_thread_stack_n_times(thread_ptr, 1) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::SetConstAttribute(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(value_cell) => match env.get_thread_latest_binded_stated_cell(thread_ptr, 1) {
                    Ok(object_cell) => {
                        let object_ptr = match object_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(object_ptr) {
                            Some(PengHeapValue::Object(object)) => {
                                object.fields.insert(name_ptr, value);
                            }

                            Some(PengHeapValue::Type(PengType::Custom(custom_type))) => {
                                custom_type.fields.insert(name_ptr, value);
                            }

                            Some(PengHeapValue::Type(_)) => {
                                return Err(PengError::ExpectedType);
                            }

                            Some(_) => {
                                return Err(PengError::ExpectedObject);
                            }

                            None => {
                                return Err(PengError::HeapValueNotFound(object_ptr));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GetConstAttribute(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(object_cell) => {
                    let object_ptr = match object_cell.value() {
                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                        PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                        PengStated::Uninitialized => {
                            return Err(PengError::CannotReadUninitialized);
                        }
                    };

                    let value = match env.get_heap(object_ptr) {
                        Some(PengHeapValue::Object(object)) => match object.fields.get(&name_ptr) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(PengError::AttributeNotFound(name_ptr));
                            }
                        },

                        Some(PengHeapValue::Type(PengType::Custom(custom_type))) => {
                            match custom_type.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::AttributeNotFound(name_ptr));
                                }
                            }
                        }

                        Some(PengHeapValue::Type(_)) => {
                            return Err(PengError::ExpectedType);
                        }

                        Some(_) => {
                            return Err(PengError::ExpectedObject);
                        }

                        None => {
                            return Err(PengError::HeapValueNotFound(object_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => match env.push_thread_binded_stated_cell(thread_ptr, value) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        },

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::SetConstMember(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(value_cell) => match env.get_thread_latest_binded_stated_cell(thread_ptr, 1) {
                    Ok(module_cell) => {
                        let module_ptr = match module_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(module_ptr) {
                            Some(PengHeapValue::Module(module)) => {
                                module.members.insert(name_ptr, value);
                            }

                            Some(_) => {
                                return Err(PengError::ExpectedModule);
                            }

                            None => {
                                return Err(PengError::HeapValueNotFound(module_ptr));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                },

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GetConstMember(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(module_cell) => {
                    let module_ptr = match module_cell.value() {
                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                        PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                        PengStated::Uninitialized => {
                            return Err(PengError::CannotReadUninitialized);
                        }
                    };

                    let value = match env.get_heap(module_ptr) {
                        Some(PengHeapValue::Module(module)) => {
                            match module.members.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::AttributeNotFound(name_ptr));
                                }
                            }
                        }

                        Some(_) => {
                            return Err(PengError::ExpectedModule);
                        }

                        None => {
                            return Err(PengError::HeapValueNotFound(module_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => match env.push_thread_binded_stated_cell(thread_ptr, value) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        },

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Jump(target) => match env.set_thread_program_counter(thread_ptr, target) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::JumpIfTrue(address) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(condition_cell) => {
                    let condition = match condition_cell.value() {
                        PengStated::Initialized(PengCell::Bool(value)) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            if condition {
                                match env.set_thread_program_counter(thread_ptr, address) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::JumpIfFalse(address) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(condition_cell) => {
                    let condition = match condition_cell.value() {
                        PengStated::Initialized(PengCell::Bool(value)) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            if !condition {
                                match env.set_thread_program_counter(thread_ptr, address) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Return => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(value) => {
                    return Ok(Some(value));
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::FunctionCall(args_count) => match env.get_thread_stack_len(thread_ptr) {
            Ok(stack_len) => {
                if stack_len < args_count + 1 {
                    return Err(PengError::TooFewArguments {
                        expected: args_count + 1,
                        found: stack_len,
                    });
                }

                let function_index = stack_len - args_count - 1;

                match env.get_thread_latest_binded_stated_cell(thread_ptr, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            PengStated::Initialized(_) => return Err(PengError::ExpectedReference),
                            PengStated::Uninitialized => {
                                return Err(PengError::CannotReadUninitialized);
                            }
                        };

                        let function = match env.get_heap(function_ptr) {
                            Some(PengHeapValue::Function(function)) => function.clone(),
                            Some(_) => return Err(PengError::ExpectedFunction),
                            None => return Err(PengError::HeapValueNotFound(function_ptr)),
                        };

                        match function {
                            PengFunction::Native(_) => {
                                match env.pop_thread_stack_at(thread_ptr, args_count) {
                                    Ok(_) => {
                                        match env.push_thread_frame(
                                            thread_ptr,
                                            PengFrame::new(
                                                function_ptr,
                                                function_index,
                                                args_count,
                                            ),
                                        ) {
                                            Ok(()) => return Ok(None),
                                            Err(e) => {
                                                return Err(e.push(PengError::InvalidInstruction(
                                                    instruction,
                                                )));
                                            }
                                        }
                                    }

                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }

                            PengFunction::Bytecode(func_btc) => match func_btc.params {
                                PengBytecodeFunctionParams::Fixed(expected_count) => {
                                    if args_count != expected_count {
                                        return Err(PengError::TooFewArguments {
                                            expected: expected_count,
                                            found: args_count,
                                        });
                                    }

                                    match env.pop_thread_stack_at(thread_ptr, args_count) {
                                        Ok(_) => {
                                            match env.push_thread_frame(
                                                thread_ptr,
                                                PengFrame::new(
                                                    function_ptr,
                                                    function_index,
                                                    expected_count,
                                                ),
                                            ) {
                                                Ok(()) => return Ok(None),
                                                Err(e) => {
                                                    return Err(e.push(
                                                        PengError::InvalidInstruction(instruction),
                                                    ));
                                                }
                                            }
                                        }

                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengBytecodeFunctionParams::Variadic(fixed_count) => {
                                    if args_count < fixed_count {
                                        return Err(PengError::TooFewArguments {
                                            expected: fixed_count,
                                            found: args_count,
                                        });
                                    }

                                    let args = match env
                                        .get_thread_latest_n_binded_stated_cells_cloned(
                                            thread_ptr, args_count,
                                        ) {
                                        Ok(args) => args,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    };

                                    match env.pop_thread_stack_n_times(thread_ptr, args_count + 1) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }

                                    for i in 0..fixed_count {
                                        match env.push_thread_binded_stated_cell(
                                            thread_ptr,
                                            args[i].clone(),
                                        ) {
                                            Ok(()) => {}
                                            Err(e) => {
                                                return Err(e.push(PengError::InvalidInstruction(
                                                    instruction,
                                                )));
                                            }
                                        }
                                    }

                                    let rest = args[fixed_count..].to_vec();

                                    let vector_ptr = env.create_heap_value(PengHeapValue::Vector(
                                        PengVector::new(rest),
                                    ));

                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Reference(vector_ptr),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }

                                    match env.push_thread_frame(
                                        thread_ptr,
                                        PengFrame::new(
                                            function_ptr,
                                            function_index,
                                            fixed_count + 1,
                                        ),
                                    ) {
                                        Ok(()) => return Ok(None),
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }
                            },
                        }
                    }

                    Err(e) => {
                        return Err(e.push(PengError::InvalidInstruction(instruction)));
                    }
                }
            }

            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::PushString(name_ptr) => {
            let heap_ptr =
                env.create_heap_value(PengHeapValue::String(match env.get_pooled_name(name_ptr) {
                    Some(name) => name.clone(),
                    None => {
                        return Err(PengError::NameNotFound(name_ptr));
                    }
                }));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Immutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateUnion(count) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread_ptr, count) {
                Ok(cells) => {
                    let mut values = Vec::new();

                    for cell in cells {
                        match cell.value() {
                            PengStated::Initialized(cell) => {
                                match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => match value {
                                        PengValue::Heap(PengHeapValue::Type(value)) => {
                                            values.push(value);
                                        }

                                        _ => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    },

                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }
                            }

                            PengStated::Uninitialized => {
                                return Err(PengError::InvalidInstruction(instruction));
                            }
                        }
                    }

                    match env.pop_thread_stack_n_times(thread_ptr, count) {
                        Ok(()) => {
                            let heap_ptr = env.create_heap_value(PengHeapValue::Union(PengUnion {
                                unions: values,
                            }));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Convert => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(target_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(value_cell) => {
                            let target_value = match target_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::CannotReadUninitialized);
                                }
                            };

                            let value = match value_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let target_type = match target_value {
                                PengValue::Heap(PengHeapValue::Type(value)) => value,
                                _ => return Err(PengError::ExpectedType),
                            };

                            let converted = match value.convert(target_type) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            let cell = match env.get_cell_from_value(converted) {
                                Ok(cell) => cell,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(cell)),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Add => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    match aaa.checked_add(*bbb) {
                                        Some(v) => PengCell::Int(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_add(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_add(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa + bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa + bbb)
                                }
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Subtract => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    match aaa.checked_sub(*bbb) {
                                        Some(v) => PengCell::Int(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_sub(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_sub(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa - bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa - bbb)
                                }
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Multiply => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    match aaa.checked_mul(*bbb) {
                                        Some(v) => PengCell::Int(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_mul(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_mul(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa * bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa * bbb)
                                }
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Divide => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    match aaa.checked_div(*bbb) {
                                        Some(v) => PengCell::Int(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_div(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    if *bbb == 0 {
                                        return Err(PengError::InvalidInstruction(instruction));
                                    }

                                    PengCell::Byte(*aaa / *bbb)
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa / bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa / bbb)
                                }
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Power => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    if *bbb < 0 {
                                        PengCell::Float64((*aaa as f64).powf(*bbb as f64))
                                    } else {
                                        PengCell::Int(aaa.pow(*bbb as u32))
                                    }
                                }

                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    PengCell::Uint(aaa.pow(*bbb as u32))
                                }

                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_pow(*bbb as u32) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }

                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa.powf(*bbb))
                                }

                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa.powf(*bbb))
                                }

                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Remainder => {
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread_ptr) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (PengStated::Initialized(aa), PengStated::Initialized(bb)) => {
                            match (aa, bb) {
                                (PengCell::Int(aaa), PengCell::Int(bbb)) => {
                                    match aaa.checked_rem(*bbb) {
                                        Some(v) => PengCell::Int(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_rem(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    if *bbb == 0 {
                                        return Err(PengError::InvalidInstruction(instruction));
                                    }

                                    PengCell::Byte(*aaa % *bbb)
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa % bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa % bbb)
                                }
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }
        PengInstruction::Negate => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        PengStated::Initialized(cell) => match cell {
                            PengCell::Int(v) => PengCell::Int(-v),
                            PengCell::Float32(v) => PengCell::Float32(-v),
                            PengCell::Float64(v) => PengCell::Float64(-v),
                            _ => {
                                return Err(PengError::InvalidInstruction(instruction));
                            }
                        },

                        PengStated::Uninitialized => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(value)),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Concat => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let value = match (left, right) {
                                (
                                    PengValue::Heap(PengHeapValue::String(a)),
                                    PengValue::Heap(PengHeapValue::String(b)),
                                ) => PengValue::Heap(PengHeapValue::String(format!("{}{}", a, b))),

                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let cell = match env.get_cell_from_value(value) {
                                Ok(cell) => cell,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(cell)),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::And => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(left && right),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Or => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(left || right),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Not => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        PengStated::Initialized(PengCell::Bool(value)) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Bool(
                                    !value,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::Equals => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(left.equals(&right)),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::NotEquals => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(!left.equals(&right)),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::TryFunctionCall(args_count) => {
            match env.execute_try_function_call(thread_ptr, args_count) {
                Ok(()) => return Ok(None),
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GreaterThan => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let result = match left.greater_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(result),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GreaterEqualsThan => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let result = match left.greater_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(result),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::LessThan => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let result = match left.less_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(result),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::LessEqualsThan => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let result = match left.less_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(
                                            PengCell::Bool(result),
                                        )),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::OperationCall => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread_ptr, 2)
                                .cloned()
                            {
                                Ok(operation_cell) => {
                                    let operation_ptr = match operation_cell.value() {
                                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                                        PengStated::Initialized(_) => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                        PengStated::Uninitialized => {
                                            return Err(PengError::CannotReadUninitialized);
                                        }
                                    };

                                    match env.get_heap(operation_ptr) {
                                        Some(PengHeapValue::Operation(_)) => {}

                                        Some(PengHeapValue::Function(_)) => {
                                            return Err(PengError::ExpectedOperation);
                                        }

                                        Some(_) => {
                                            return Err(PengError::ExpectedOperation);
                                        }

                                        None => {
                                            return Err(PengError::HeapValueNotFound(
                                                operation_ptr,
                                            ));
                                        }
                                    }

                                    match env.pop_thread_stack_n_times(thread_ptr, 3) {
                                        Ok(()) => {
                                            match env.get_thread_stack_len(thread_ptr) {
                                                Ok(base) => {
                                                    match env.push_thread_binded_stated_cell(
                                                        thread_ptr, left_cell,
                                                    ) {
                                                        Ok(()) => {
                                                            match env
                                                                .push_thread_binded_stated_cell(
                                                                    thread_ptr, right_cell,
                                                                ) {
                                                                Ok(()) => {
                                                                    match env.push_thread_frame(
                                                                        thread_ptr,
                                                                        PengFrame::new(
                                                                            operation_ptr,
                                                                            base,
                                                                            2,
                                                                        ),
                                                                    ) {
                                                                        Ok(()) => return Ok(None),
                                                                        Err(e) => {
                                                                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                                                                        }
                                                                    }
                                                                }

                                                                Err(e) => {
                                                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                                                }
                                                            }
                                                        }

                                                        Err(e) => {
                                                            return Err(e.push(
                                                                PengError::InvalidInstruction(
                                                                    instruction,
                                                                ),
                                                            ));
                                                        }
                                                    }
                                                }

                                                Err(e) => {
                                                    return Err(e.push(
                                                        PengError::InvalidInstruction(instruction),
                                                    ));
                                                }
                                            }
                                        }

                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GetIndex => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(index_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(object_cell) => {
                            let index_value = match index_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let index = match index_value {
                                PengValue::Cell(PengCell::Int(v)) => {
                                    if v < 0 {
                                        return Err(PengError::InvalidIndexType(format!(
                                            "{:?}",
                                            index_value
                                        )));
                                    }

                                    v as usize
                                }

                                PengValue::Cell(PengCell::Uint(v)) => v,

                                _ => {
                                    return Err(PengError::InvalidIndexType(format!(
                                        "{:?}",
                                        index_value
                                    )));
                                }
                            };

                            let object_ptr = match object_cell.value() {
                                PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                                PengStated::Initialized(_) => {
                                    return Err(PengError::ExpectedReference);
                                }
                                PengStated::Uninitialized => {
                                    return Err(PengError::CannotReadUninitialized);
                                }
                            };

                            let value = match env.get_heap(object_ptr) {
                                Some(PengHeapValue::Vector(vector)) => {
                                    match vector.values.get(index) {
                                        Some(value) => value.clone(),
                                        None => {
                                            return Err(PengError::IndexOutOfBounds {
                                                index,
                                                len: vector.values.len(),
                                            });
                                        }
                                    }
                                }

                                Some(_) => {
                                    return Err(PengError::CannotIndexValue(
                                        "non-vector heap value".to_string(),
                                    ));
                                }

                                None => {
                                    return Err(PengError::HeapValueNotFound(object_ptr));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(thread_ptr, value) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::SetIndex => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(value_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(index_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread_ptr, 2)
                                .cloned()
                            {
                                Ok(object_cell) => {
                                    let index_value = match index_cell.value() {
                                        PengStated::Initialized(cell) => {
                                            match env.get_value_from_cell(cell.clone()) {
                                                Ok(value) => value,
                                                Err(e) => {
                                                    return Err(e.push(
                                                        PengError::InvalidInstruction(instruction),
                                                    ));
                                                }
                                            }
                                        }

                                        PengStated::Uninitialized => {
                                            return Err(PengError::InvalidInstruction(instruction));
                                        }
                                    };

                                    let index = match index_value {
                                        PengValue::Cell(PengCell::Int(v)) => {
                                            if v < 0 {
                                                return Err(PengError::InvalidIndexType(format!(
                                                    "{:?}",
                                                    index_value
                                                )));
                                            }

                                            v as usize
                                        }

                                        PengValue::Cell(PengCell::Uint(v)) => v,

                                        _ => {
                                            return Err(PengError::InvalidIndexType(format!(
                                                "{:?}",
                                                index_value
                                            )));
                                        }
                                    };

                                    let object_ptr = match object_cell.value() {
                                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                                        PengStated::Initialized(_) => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                        PengStated::Uninitialized => {
                                            return Err(PengError::CannotReadUninitialized);
                                        }
                                    };

                                    match env.get_heap_mut(object_ptr) {
                                        Some(PengHeapValue::Vector(vector)) => {
                                            if index >= vector.values.len() {
                                                return Err(PengError::IndexOutOfBounds {
                                                    index,
                                                    len: vector.values.len(),
                                                });
                                            }

                                            vector.values[index] = value_cell;
                                        }

                                        Some(_) => {
                                            return Err(PengError::CannotSetIndex(
                                                "non-vector heap value".to_string(),
                                            ));
                                        }

                                        None => {
                                            return Err(PengError::HeapValueNotFound(object_ptr));
                                        }
                                    }

                                    match env.pop_thread_stack_n_times(thread_ptr, 3) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::FunctionCallSpread(fixed_args_count) => {
            match env.get_thread_stack_len(thread_ptr) {
                Ok(stack_len) => {
                    if stack_len < fixed_args_count + 2 {
                        return Err(PengError::TooFewArguments {
                            expected: fixed_args_count + 2,
                            found: stack_len,
                        });
                    }

                    let spread_cell = match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                        .cloned()
                    {
                        Ok(cell) => cell,
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    };

                    let spread_ptr = match spread_cell.value() {
                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                        PengStated::Initialized(_) => {
                            return Err(PengError::ExpectedReference);
                        }
                        PengStated::Uninitialized => {
                            return Err(PengError::CannotReadUninitialized);
                        }
                    };

                    let spread_values = match env.get_heap(spread_ptr) {
                        Some(PengHeapValue::Vector(vector)) => vector.values.clone(),
                        Some(_) => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                        None => {
                            return Err(PengError::HeapValueNotFound(spread_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }

                    for value in &spread_values {
                        match env.push_thread_binded_stated_cell(thread_ptr, value.clone()) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    let args_count = match fixed_args_count.checked_add(spread_values.len()) {
                        Some(value) => value,
                        None => return Err(PengError::ArithmeticOverflow),
                    };

                    match execute_instruction(
                        PengInstruction::FunctionCall(args_count),
                        None,
                        thread_ptr,
                        _frame_base,
                        env,
                    ) {
                        Ok(value) => return Ok(value),
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::TryFunctionCallSpread(fixed_args_count) => {
            match env.get_thread_stack_len(thread_ptr) {
                Ok(stack_len) => {
                    if stack_len < fixed_args_count + 2 {
                        return Err(PengError::TooFewArguments {
                            expected: fixed_args_count + 2,
                            found: stack_len,
                        });
                    }

                    let spread_cell = match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                        .cloned()
                    {
                        Ok(cell) => cell,
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    };

                    let spread_ptr = match spread_cell.value() {
                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                        PengStated::Initialized(_) => {
                            return Err(PengError::ExpectedReference);
                        }
                        PengStated::Uninitialized => {
                            return Err(PengError::CannotReadUninitialized);
                        }
                    };

                    let spread_values = match env.get_heap(spread_ptr) {
                        Some(PengHeapValue::Vector(vector)) => vector.values.clone(),
                        Some(_) => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                        None => {
                            return Err(PengError::HeapValueNotFound(spread_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }

                    for value in &spread_values {
                        match env.push_thread_binded_stated_cell(thread_ptr, value.clone()) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    let args_count = match fixed_args_count.checked_add(spread_values.len()) {
                        Some(value) => value,
                        None => return Err(PengError::ArithmeticOverflow),
                    };

                    match env.execute_try_function_call(thread_ptr, args_count) {
                        Ok(()) => return Ok(None),
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::TryOperationCall => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread_ptr, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread_ptr, 2)
                                .cloned()
                            {
                                Ok(operation_cell) => {
                                    let operation_ptr = match operation_cell.value() {
                                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,

                                        PengStated::Initialized(_) => {
                                            return Err(PengError::ExpectedReference);
                                        }

                                        PengStated::Uninitialized => {
                                            return Err(PengError::CannotReadUninitialized);
                                        }
                                    };

                                    match env.get_heap(operation_ptr) {
                                        Some(PengHeapValue::Operation(_)) => {}

                                        Some(_) => {
                                            return Err(PengError::ExpectedOperation);
                                        }

                                        None => {
                                            return Err(PengError::HeapValueNotFound(
                                                operation_ptr,
                                            ));
                                        }
                                    }

                                    match env.pop_thread_stack_n_times(thread_ptr, 3) {
                                        Ok(()) => match env.get_thread_stack_len(thread_ptr) {
                                            Ok(base) => {
                                                match env.push_thread_binded_stated_cell(
                                                    thread_ptr, left_cell,
                                                ) {
                                                    Ok(()) => {
                                                        match env.push_thread_binded_stated_cell(
                                                            thread_ptr, right_cell,
                                                        ) {
                                                            Ok(()) => {
                                                                match env.push_thread_frame(
                                                                    thread_ptr,
                                                                    PengFrame::new_try(
                                                                        operation_ptr,
                                                                        base,
                                                                        2,
                                                                    ),
                                                                ) {
                                                                    Ok(()) => return Ok(None),

                                                                    Err(e) => {
                                                                        return Err(e.push(
                                                                        PengError::InvalidInstruction(
                                                                            instruction,
                                                                        ),
                                                                    ));
                                                                    }
                                                                }
                                                            }

                                                            Err(e) => {
                                                                return Err(e.push(
                                                                    PengError::InvalidInstruction(
                                                                        instruction,
                                                                    ),
                                                                ));
                                                            }
                                                        }
                                                    }

                                                    Err(e) => {
                                                        return Err(e.push(
                                                            PengError::InvalidInstruction(
                                                                instruction,
                                                            ),
                                                        ));
                                                    }
                                                }
                                            }

                                            Err(e) => {
                                                return Err(e.push(PengError::InvalidInstruction(
                                                    instruction,
                                                )));
                                            }
                                        },

                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            }
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }
    }
    Ok(None)
}
