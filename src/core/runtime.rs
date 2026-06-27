use crate::core::*;

pub fn step_thread(
    env: &mut PengEnv,
    thread: PengHeapPtr,
) -> Result<Option<PengBindedCell>, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, frame_params_count) =
        match env.get_heap(thread) {
            Some(PengValue::Box(PengBox::Thread(thread))) => {
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
                return Err(PengError::ThreadNotFound(thread));
            }
        };

    let (ntv_opt, should_end_frame, instruction, constant): (
        Option<PengNativeCallable>,
        bool,
        PengInstruction,
        Option<PengValue>,
    ) = match env.get_heap(frame_function_ptr) {
        Some(PengValue::Box(PengBox::Function(func))) => match func {
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

            PengFunction::Native(func_ntv) => (
                Some(PengNativeCallable::Function(func_ntv.clone())),
                false,
                PengInstruction::Add,
                None,
            ),
        },
        Some(PengValue::Box(PengBox::Operation(operation))) => match operation {
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

            PengOperation::Native(operation_ntv) => (
                Some(PengNativeCallable::Operation(operation_ntv.clone())),
                false,
                PengInstruction::Add,
                None,
            ),
        },
        Some(_) => {
            return Err(PengError::ExpectedFunction);
        }
        None => {
            return Err(PengError::HeapValueNotFound(frame_function_ptr));
        }
    };

    match ntv_opt {
        Some(ntv_call) => {
            let args = match env
                .get_thread_latest_n_binded_stated_cells_cloned(thread, frame_params_count)
            {
                Ok(args) => {
                    let mut fargs: Vec<PengBindedCell> = Vec::new();
                    for a in args {
                        match a {
                            PengBinded::Immutable(s) => fargs.push(PengBinded::Immutable(s)),
                            PengBinded::Mutable(s) => fargs.push(PengBinded::Mutable(s)),
                        }
                    }
                    fargs
                }
                Err(e) => {
                    return Err(e.push(PengError::ExpectedFunction));
                }
            };

            let ret = match ntv_call {
                PengNativeCallable::Function(ntv_fn) => match ntv_fn.call(args, env) {
                    Ok(ret) => ret,
                    Err(e) => {
                        return Err(e.push(PengError::CannotCallValue(
                            "native function call failed".to_string(),
                        )));
                    }
                },
                PengNativeCallable::Operation(ntv_oper) => {
                    if args.len() != 2 {
                        return Err(PengError::TooFewArguments {
                            expected: 2,
                            found: args.len(),
                        });
                    }

                    match ntv_oper.call((args[0].clone(), args[1].clone()), env) {
                        Ok(ret) => ret,
                        Err(e) => {
                            return Err(e.push(PengError::CannotCallValue(
                                "native operation call failed".to_string(),
                            )));
                        }
                    }
                }
            };

            let frame = match env.pop_thread_frame(thread) {
                Ok(frame) => frame,
                Err(e) => {
                    return Err(e.push(PengError::ExpectedFunction));
                }
            };

            match env.truncate_thread_stack(thread, frame.base) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed to truncate stack after native function call".to_string(),
                    )));
                }
            }

            match env.get_thread_frames_len(thread) {
                Ok(frames_len) => {
                    let r = match ret {
                        PengBinded::Immutable(c) => PengBinded::Immutable(c),
                        PengBinded::Mutable(c) => PengBinded::Mutable(c),
                    };
                    if frames_len == 0 {
                        return Ok(Some(r));
                    }

                    match env.push_thread_binded_stated_cell(thread, r) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed to push native function return value".to_string(),
                            )));
                        }
                    }

                    if frame.is_try {
                        match env.push_thread_binded_stated_cell(
                            thread,
                            PengBinded::Mutable(PengCell::Bool(true)),
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

            match env.get_heap_mut(thread) {
                Some(PengValue::Box(PengBox::Thread(thread))) => {
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
                None => return Err(PengError::ThreadNotFound(thread)),
            }

            match execute_instruction(instruction.clone(), constant, thread, frame_base, env) {
                Ok(res) => match res {
                    Some(ret) => {
                        let frame = match env.pop_thread_frame(thread) {
                            Ok(frame) => frame,
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        };

                        match env.truncate_thread_stack(thread, frame.base) {
                            Ok(()) => {}
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }

                        match env.get_thread_frames_len(thread) {
                            Ok(frames_len) => {
                                if frames_len == 0 {
                                    return Ok(Some(ret));
                                }

                                match env.push_thread_binded_stated_cell(thread, ret) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                }

                                if frame.is_try {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(true)),
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

                Err(e) => match env.recover_thread_try_error(thread) {
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

enum PengNativeCallable {
    Function(PengNativeFunction),
    Operation(PengNativeOperation),
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: Option<PengValue>,
    thread: PengHeapPtr,
    frame_base: usize,
    env: &mut PengEnv,
) -> Result<Option<PengBindedCell>, PengError> {
    match instruction {
        PengInstruction::MakeImmutable => {
            let cell = match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(cell) => cell,
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            };

            let immutable = match cell {
                PengBinded::Mutable(value) => PengBinded::Immutable(value),
                PengBinded::Immutable(value) => PengBinded::Immutable(value),
            };

            match env.pop_thread_stack_n_times(thread, 1) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }

            match env.push_thread_binded_stated_cell(thread, immutable) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::PushConst(_) => {
            let constant = match constant {
                Some(value) => value,
                None => return Err(PengError::InstructionExpectedConstant),
            };

            match constant {
                PengValue::Cell(cell) => {
                    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell))
                    {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                PengValue::Box(value) => {
                    let ptr = env.create_heap_value(PengValue::Box(value));
                    let cell = PengCell::Reference(ptr);

                    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell))
                    {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
            }
        }

        PengInstruction::PushLocal(local) => match env.get_thread_local(thread, local).cloned()
        {
            Ok(cell) => match env.push_thread_binded_stated_cell(thread, cell.clone()) {
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
            let stack_len = match env.get_thread_stack_len(thread) {
                Ok(len) => len,
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            };

            if stack_len == 0 {
                return Err(PengError::InvalidInstruction(instruction));
            }

            let value_index = stack_len - 1;

            let target_index = match frame_base.checked_add(local) {
                Some(index) => index,
                None => return Err(PengError::InvalidFrame),
            };

            // declaração local: o topo da stack já é o próprio slot
            if value_index == target_index {
                return Ok(None);
            }

            let current = match env.get_thread_local(thread, local).cloned() {
                Ok(current) => current,
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            };

            match current {
                PengBinded::Immutable(_) => {
                    return Err(PengError::CannotMutateImmutable);
                }

                PengBinded::Mutable(_) => {}
            }

            let value = match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(value) => value,
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            };

            match env.set_thread_local(thread, local, value) {
                Ok(()) => {}
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            }

            match env.pop_thread_stack_n_times(thread, 1) {
                Ok(()) => {}
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            }
        }

        PengInstruction::PushHeap(ptr) => {
            let value = match env.get_heap(ptr) {
                Some(value) => value.clone(),
                None => return Err(PengError::HeapValueNotFound(ptr)),
            };

            let cell = match env.get_cell_from_value(value) {
                Ok(cell) => cell,
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            };

            match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell)) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::PushHeapRef(ptr) => {
            match env.push_thread_binded_stated_cell(
                thread,
                PengBinded::Mutable(PengCell::Reference(ptr)),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::StoreHeap => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(value_cell) => match env
                    .get_thread_latest_binded_stated_cell(thread, 1)
                    .cloned()
                {
                    Ok(ref_cell) => {
                        let ptr: PengHeapPtr = match ref_cell.value() {
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
                        };

                        let value = match env.get_value_from_cell(value_cell.value().clone()) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        };

                        match env.pop_thread_stack_n_times(thread, 2) {
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
            let heap_ptr =
                env.create_heap_value(PengValue::Box(PengBox::Object(PengObject::new_empty())));

            match env.push_thread_binded_stated_cell(
                thread,
                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateEmptyModule => {
            let heap_ptr =
                env.create_heap_value(PengValue::Box(PengBox::Module(PengModule::new_empty())));

            match env.push_thread_binded_stated_cell(
                thread,
                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateVector(size) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread, size) {
                Ok(cells) => match env.pop_thread_stack_n_times(thread, size) {
                    Ok(()) => {
                        let heap_ptr = env.create_heap_value(PengValue::Box(PengBox::Vector(
                            PengVector::new(cells),
                        )));

                        match env.push_thread_binded_stated_cell(
                            thread,
                            PengBinded::Mutable(PengCell::Reference(heap_ptr)),
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
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread, types_count) {
                Ok(cells) => {
                    let mut custom_types = Vec::new();

                    for cell in cells {
                        match env.get_value_from_cell(cell.value().clone()) {
                            Ok(value) => match value {
                                PengValue::Box(PengBox::Type(PengType::Custom(custom_type))) => {
                                    custom_types.push(custom_type);
                                }

                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            },

                            Err(e) => {
                                return Err(e.push(PengError::InvalidInstruction(instruction)));
                            }
                        }
                    }

                    match env.pop_thread_stack_n_times(thread, types_count) {
                        Ok(()) => {
                            let heap_ptr = env.create_heap_value(PengValue::Box(PengBox::Type(
                                PengType::Custom(PengCustomType::new_super_type(custom_types)),
                            )));

                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(cell) => {
                    let custom_type = match env.get_value_from_cell(cell.value().clone()) {
                        Ok(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
                            custom_type
                        }

                        Ok(_) => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }

                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            let heap_ptr = env.create_heap_value(PengValue::Box(PengBox::Object(
                                PengObject::new(custom_type.fields),
                            )));

                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(cell) => match env.push_thread_binded_stated_cell(thread, cell.clone()) {
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

        PengInstruction::Pop => match env.pop_thread_stack_n_times(thread, 1) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::SetAttribute(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(value_cell) => match env.get_thread_latest_binded_stated_cell(thread, 1) {
                    Ok(object_cell) => {
                        match ensure_mutable_base(object_cell) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                        let object_ptr = match object_cell.value() {
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(object_ptr) {
                            Some(PengValue::Box(PengBox::Object(object))) => {
                                object.fields.insert(name_ptr, value);
                            }

                            Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
                                custom_type.fields.insert(name_ptr, value);
                            }

                            Some(PengValue::Box(PengBox::Type(_))) => {
                                return Err(PengError::ExpectedType);
                            }

                            Some(_) => {
                                return Err(PengError::ExpectedObject);
                            }

                            None => {
                                return Err(PengError::HeapValueNotFound(object_ptr));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread, 2) {
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

        PengInstruction::GetAttribute(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(object_cell) => {
                    let object_ptr = match object_cell.value() {
                        PengCell::Reference(ptr) => *ptr,
                        _ => return Err(PengError::ExpectedReference),
                    };

                    let value = match env.get_heap(object_ptr) {
                        Some(PengValue::Box(PengBox::Object(object))) => {
                            match object.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::AttributeNotFound(name_ptr));
                                }
                            }
                        }

                        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
                            match custom_type.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::AttributeNotFound(name_ptr));
                                }
                            }
                        }

                        Some(PengValue::Box(PengBox::Type(_))) => {
                            return Err(PengError::ExpectedType);
                        }

                        Some(_) => {
                            return Err(PengError::ExpectedObject);
                        }

                        None => {
                            return Err(PengError::HeapValueNotFound(object_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            let value = inherit_binding_from_base(&object_cell, value);

                            match env.push_thread_binded_stated_cell(thread, value) {
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

        PengInstruction::SetMember(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(value_cell) => match env.get_thread_latest_binded_stated_cell(thread, 1) {
                    Ok(module_cell) => {
                        match ensure_mutable_base(module_cell) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                        let module_ptr = match module_cell.value() {
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(module_ptr) {
                            Some(PengValue::Box(PengBox::Module(module))) => {
                                module.members.insert(name_ptr, value);
                            }

                            Some(_) => {
                                return Err(PengError::ExpectedModule);
                            }

                            None => {
                                return Err(PengError::HeapValueNotFound(module_ptr));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread, 2) {
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

        PengInstruction::GetMember(name_ptr) => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(module_cell) => {
                    let module_ptr = match module_cell.value() {
                        PengCell::Reference(ptr) => *ptr,
                        _ => return Err(PengError::ExpectedReference),
                    };

                    let value = match env.get_heap(module_ptr) {
                        Some(PengValue::Box(PengBox::Module(module))) => {
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

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            let value = inherit_binding_from_base(&module_cell, value);
                            match env.push_thread_binded_stated_cell(thread, value) {
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

        PengInstruction::Jump(target) => match env.set_thread_program_counter(thread, target) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidInstruction(instruction)));
            }
        },

        PengInstruction::JumpIfTrue(address) => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(condition_cell) => {
                    let condition = match condition_cell.value() {
                        PengCell::Bool(value) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            if condition {
                                match env.set_thread_program_counter(thread, address) {
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(condition_cell) => {
                    let condition = match condition_cell.value() {
                        PengCell::Bool(value) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            if !condition {
                                match env.set_thread_program_counter(thread, address) {
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
                .get_thread_latest_binded_stated_cell(thread, 0)
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

        PengInstruction::FunctionCall(args_count) => match env.get_thread_stack_len(thread) {
            Ok(stack_len) => {
                if stack_len < args_count + 1 {
                    return Err(PengError::TooFewArguments {
                        expected: args_count + 1,
                        found: stack_len,
                    });
                }

                let function_index = stack_len - args_count - 1;

                match env.get_thread_latest_binded_stated_cell(thread, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengCell::Reference(ptr) => *ptr,
                            _ => return Err(PengError::ExpectedReference),
                        };

                        let function = match env.get_heap(function_ptr) {
                            Some(PengValue::Box(PengBox::Function(function))) => function.clone(),
                            Some(_) => return Err(PengError::ExpectedFunction),
                            None => return Err(PengError::HeapValueNotFound(function_ptr)),
                        };

                        match function {
                            PengFunction::Native(_) => {
                                match env.pop_thread_stack_at(thread, args_count) {
                                    Ok(_) => {
                                        match env.push_thread_frame(
                                            thread,
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

                                    match env.pop_thread_stack_at(thread, args_count) {
                                        Ok(_) => {
                                            match env.push_thread_frame(
                                                thread,
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
                                            thread, args_count,
                                        ) {
                                        Ok(args) => args,
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    };

                                    match env.pop_thread_stack_n_times(thread, args_count + 1) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }

                                    for i in 0..fixed_count {
                                        match env.push_thread_binded_stated_cell(
                                            thread,
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

                                    let vector_ptr = env.create_heap_value(PengValue::Box(
                                        PengBox::Vector(PengVector::new(rest)),
                                    ));

                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Reference(vector_ptr)),
                                    ) {
                                        Ok(()) => {}
                                        Err(e) => {
                                            return Err(
                                                e.push(PengError::InvalidInstruction(instruction))
                                            );
                                        }
                                    }

                                    match env.push_thread_frame(
                                        thread,
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
            let heap_ptr = env.create_heap_value(PengValue::Box(PengBox::String(
                match env.get_pooled_name(name_ptr) {
                    Some(name) => name.clone(),
                    None => {
                        return Err(PengError::NameNotFound(name_ptr));
                    }
                },
            )));

            match env.push_thread_binded_stated_cell(
                thread,
                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::CreateUnion(count) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread, count) {
                Ok(cells) => {
                    let mut values = Vec::new();

                    for cell in cells {
                        match cell.value() {
                            cell => match env.get_value_from_cell(cell.clone()) {
                                Ok(value) => match value {
                                    PengValue::Box(PengBox::Type(value)) => {
                                        values.push(value);
                                    }

                                    _ => {
                                        return Err(PengError::InvalidInstruction(instruction));
                                    }
                                },

                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            },
                        }
                    }

                    match env.pop_thread_stack_n_times(thread, count) {
                        Ok(()) => {
                            let heap_ptr =
                                env.create_heap_value(PengValue::Box(PengBox::Union(PengUnion {
                                    unions: values,
                                })));

                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(PengCell::Reference(heap_ptr)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(target_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(value_cell) => {
                            let target_value = match target_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let value = match value_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let target_type = match target_value {
                                PengValue::Box(PengBox::Type(value)) => value,
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

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(cell),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
            match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
                Ok((a, b)) => {
                    let res: PengCell = match (a.value(), b.value()) {
                        (aa, bb) => match (aa, bb) {
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
                        },
                    };
                    match env.pop_thread_stack_n_times(thread, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(res),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        cell => match cell {
                            PengCell::Int(v) => PengCell::Int(-v),
                            PengCell::Float32(v) => PengCell::Float32(-v),
                            PengCell::Float64(v) => PengCell::Float64(-v),
                            _ => {
                                return Err(PengError::InvalidInstruction(instruction));
                            }
                        },
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(value),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let value = match (left, right) {
                                (
                                    PengValue::Box(PengBox::String(a)),
                                    PengValue::Box(PengBox::String(b)),
                                ) => PengValue::Box(PengBox::String(format!("{}{}", a, b))),

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

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(cell),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengCell::Bool(value) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengCell::Bool(value) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(left && right)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                PengCell::Bool(value) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            let left = match left_cell.value() {
                                PengCell::Bool(value) => *value,
                                _ => {
                                    return Err(PengError::InvalidInstruction(instruction));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(left || right)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        PengCell::Bool(value) => *value,
                        _ => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread,
                                PengBinded::Mutable(PengCell::Bool(!value)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(left.equals(&right))),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(!left.equals(&right))),
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
            match env.execute_try_function_call(thread, args_count) {
                Ok(()) => return Ok(None),
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GreaterThan => {
            match env
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let result = match left.greater_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(result)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let result = match left.greater_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(result)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let result = match left.less_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(result)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            let right = match right_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let left = match left_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let result = match left.less_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread,
                                        PengBinded::Mutable(PengCell::Bool(result)),
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread, 2)
                                .cloned()
                            {
                                Ok(operation_cell) => {
                                    let operation_ptr = match operation_cell.value() {
                                        PengCell::Reference(ptr) => *ptr,
                                        _ => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                    };

                                    match env.get_heap(operation_ptr) {
                                        Some(PengValue::Box(PengBox::Operation(_))) => {}

                                        Some(PengValue::Box(PengBox::Function(_))) => {
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

                                    match env.pop_thread_stack_n_times(thread, 3) {
                                        Ok(()) => {
                                            match env.get_thread_stack_len(thread) {
                                                Ok(base) => {
                                                    match env.push_thread_binded_stated_cell(
                                                        thread, left_cell,
                                                    ) {
                                                        Ok(()) => {
                                                            match env
                                                                .push_thread_binded_stated_cell(
                                                                    thread, right_cell,
                                                                ) {
                                                                Ok(()) => {
                                                                    match env.push_thread_frame(
                                                                        thread,
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(index_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(object_cell) => {
                            let index_value = match index_cell.value() {
                                cell => match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
                                    }
                                },
                            };

                            let index = match index_value {
                                PengValue::Cell(PengCell::Int(v)) => {
                                    if v < 0 {
                                        return Err(PengError::InvalidIndexTypeValue(index_value));
                                    }

                                    v as usize
                                }

                                PengValue::Cell(PengCell::Uint(v)) => v,

                                _ => {
                                    return Err(PengError::InvalidIndexTypeValue(index_value));
                                }
                            };

                            let object_ptr = match object_cell.value() {
                                PengCell::Reference(ptr) => *ptr,
                                _ => {
                                    return Err(PengError::ExpectedReference);
                                }
                            };

                            let value = match env.get_heap(object_ptr) {
                                Some(PengValue::Box(PengBox::Vector(vector))) => {
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

                            match env.pop_thread_stack_n_times(thread, 2) {
                                Ok(()) => {
                                    let value = inherit_binding_from_base(&object_cell, value);

                                    match env.push_thread_binded_stated_cell(thread, value) {
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(value_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(index_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread, 2)
                                .cloned()
                            {
                                Ok(object_cell) => {
                                    match ensure_mutable_base(&object_cell) {
                                        Ok(()) => {}
                                        Err(e) => return Err(e),
                                    }
                                    let index_value = match index_cell.value() {
                                        cell => match env.get_value_from_cell(cell.clone()) {
                                            Ok(value) => value,
                                            Err(e) => {
                                                return Err(e.push(PengError::InvalidInstruction(
                                                    instruction,
                                                )));
                                            }
                                        },
                                    };

                                    let index = match index_value {
                                        PengValue::Cell(PengCell::Int(v)) => {
                                            if v < 0 {
                                                return Err(PengError::InvalidIndexTypeValue(
                                                    index_value,
                                                ));
                                            }

                                            v as usize
                                        }

                                        PengValue::Cell(PengCell::Uint(v)) => v,

                                        _ => {
                                            return Err(PengError::InvalidIndexTypeValue(
                                                index_value,
                                            ));
                                        }
                                    };

                                    let object_ptr = match object_cell.value() {
                                        PengCell::Reference(ptr) => *ptr,
                                        _ => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                    };

                                    match env.get_heap_mut(object_ptr) {
                                        Some(PengValue::Box(PengBox::Vector(vector))) => {
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

                                    match env.pop_thread_stack_n_times(thread, 3) {
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
            match env.get_thread_stack_len(thread) {
                Ok(stack_len) => {
                    if stack_len < fixed_args_count + 2 {
                        return Err(PengError::TooFewArguments {
                            expected: fixed_args_count + 2,
                            found: stack_len,
                        });
                    }

                    let spread_cell = match env
                        .get_thread_latest_binded_stated_cell(thread, 0)
                        .cloned()
                    {
                        Ok(cell) => cell,
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    };

                    let spread_ptr = match spread_cell.value() {
                        PengCell::Reference(ptr) => *ptr,
                        _ => {
                            return Err(PengError::ExpectedReference);
                        }
                    };

                    let spread_values = match env.get_heap(spread_ptr) {
                        Some(PengValue::Box(PengBox::Vector(vector))) => vector.values.clone(),
                        Some(_) => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                        None => {
                            return Err(PengError::HeapValueNotFound(spread_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }

                    for value in &spread_values {
                        match env.push_thread_binded_stated_cell(thread, value.clone()) {
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
                        thread,
                        frame_base,
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
            match env.get_thread_stack_len(thread) {
                Ok(stack_len) => {
                    if stack_len < fixed_args_count + 2 {
                        return Err(PengError::TooFewArguments {
                            expected: fixed_args_count + 2,
                            found: stack_len,
                        });
                    }

                    let spread_cell = match env
                        .get_thread_latest_binded_stated_cell(thread, 0)
                        .cloned()
                    {
                        Ok(cell) => cell,
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    };

                    let spread_ptr = match spread_cell.value() {
                        PengCell::Reference(ptr) => *ptr,
                        _ => {
                            return Err(PengError::ExpectedReference);
                        }
                    };

                    let spread_values = match env.get_heap(spread_ptr) {
                        Some(PengValue::Box(PengBox::Vector(vector))) => vector.values.clone(),
                        Some(_) => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                        None => {
                            return Err(PengError::HeapValueNotFound(spread_ptr));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread, 1) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }

                    for value in &spread_values {
                        match env.push_thread_binded_stated_cell(thread, value.clone()) {
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

                    match env.execute_try_function_call(thread, args_count) {
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
                .get_thread_latest_binded_stated_cell(thread, 0)
                .cloned()
            {
                Ok(right_cell) => {
                    match env
                        .get_thread_latest_binded_stated_cell(thread, 1)
                        .cloned()
                    {
                        Ok(left_cell) => {
                            match env
                                .get_thread_latest_binded_stated_cell(thread, 2)
                                .cloned()
                            {
                                Ok(operation_cell) => {
                                    let operation_ptr = match operation_cell.value() {
                                        PengCell::Reference(ptr) => *ptr,

                                        _ => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                    };

                                    match env.get_heap(operation_ptr) {
                                        Some(PengValue::Box(PengBox::Operation(_))) => {}

                                        Some(_) => {
                                            return Err(PengError::ExpectedOperation);
                                        }

                                        None => {
                                            return Err(PengError::HeapValueNotFound(
                                                operation_ptr,
                                            ));
                                        }
                                    }

                                    match env.pop_thread_stack_n_times(thread, 3) {
                                        Ok(()) => match env.get_thread_stack_len(thread) {
                                            Ok(base) => {
                                                match env.push_thread_binded_stated_cell(
                                                    thread, left_cell,
                                                ) {
                                                    Ok(()) => {
                                                        match env.push_thread_binded_stated_cell(
                                                            thread, right_cell,
                                                        ) {
                                                            Ok(()) => {
                                                                match env.push_thread_frame(
                                                                    thread,
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

fn inherit_binding_from_base(base: &PengBindedCell, value: PengBindedCell) -> PengBindedCell {
    match base {
        PengBinded::Mutable(_) => PengBinded::Mutable(value.value().clone()),
        PengBinded::Immutable(_) => PengBinded::Immutable(value.value().clone()),
    }
}

fn ensure_mutable_base(base: &PengBindedCell) -> Result<(), PengError> {
    match base {
        PengBinded::Mutable(_) => Ok(()),
        PengBinded::Immutable(_) => Err(PengError::CannotMutateImmutable),
    }
}
