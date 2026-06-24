use crate::core::*;

pub fn step_thread(
    env: &mut PengEnv,
    thread_ptr: PengHeapPtr,
) -> Result<Option<PengBindedStatedCell>, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, _frame_params_count) =
        match env.get_heap(thread_ptr) {
            Some(coloured) => {
                if let PengStated::Initialized(tval) = coloured.value.value() {
                    if let PengValue::Thread(thread) = tval {
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
                    } else {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                } else {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }
            }
            None => todo!(),
        };

    let (should_end_frame, instruction, constant) = match env.get_heap(frame_function_ptr) {
        Some(coloured) => {
            if let PengStated::Initialized(fval) = coloured.value.value() {
                if let PengValue::Function(func) = fval {
                    match func {
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
                                        Some(v) => v.clone(),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }

                                _ => PengValue::Nil,
                            };

                            (false, instr, constant)
                        }

                        PengFunction::Native(_func_ntv) => {
                            todo!()
                        }
                    }
                } else {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }
            } else {
                todo!()
            }
        }
        None => {
            return Err(PengError::Code(PengErrorCode::TestError));
        }
    };

    if should_end_frame {
        return Ok(None);
    }

    match env.get_heap_mut(thread_ptr) {
        Some(coloured) => match &mut coloured.value {
            PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                if let Some(last_frame) = thread.frames.last_mut() {
                    last_frame.program_counter = match last_frame.program_counter.checked_add(1) {
                        Some(r) => r,
                        None => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                } else {
                    return Ok(None);
                }
            }
            _ => return Err(PengError::Code(PengErrorCode::TestError)),
        },
        None => return Err(PengError::Code(PengErrorCode::TestError)),
    }

    match execute_instruction(instruction, constant, thread_ptr, frame_base, env) {
        Ok(res) => match res {
            Some(ret) => {
                let frame = match env.pop_thread_frame(thread_ptr) {
                    Ok(frame) => frame,
                    Err(e) => return Err(e),
                };

                match env.truncate_thread_stack(thread_ptr, frame.base) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }

                match env.get_thread_frames_len(thread_ptr) {
                    Ok(frames_len) => {
                        if frames_len == 0 {
                            return Ok(Some(ret));
                        }

                        match env.push_thread_binded_stated_cell(thread_ptr, ret) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }

                        if frame.is_try {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Bool(true))),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                    }

                    Err(e) => return Err(e),
                }
            }

            None => {}
        },

        Err(e) => match env.recover_thread_try_error(thread_ptr) {
            Ok(recovered) => {
                if !recovered {
                    return Err(e);
                }
            }

            Err(e) => return Err(e),
        },
    };

    Ok(None)
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    thread_ptr: PengHeapPtr,
    _frame_base: usize,
    env: &mut PengEnv,
) -> Result<Option<PengBindedStatedCell>, PengError> {
    println!("Executando: {:?}", instruction);
    match instruction {
        PengInstruction::PushConst(_) => {
            if constant.is_heap() {
                let ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                    PengStated::Initialized(constant),
                ));
                let cell = PengCell::Reference(ptr);
                match env.push_thread_binded_stated_cell(
                    thread_ptr,
                    PengBinded::Mutable(PengStated::Initialized(cell)),
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            } else {
                match constant.to_cell() {
                    Ok(cell) => {
                        match env.push_thread_binded_stated_cell(
                            thread_ptr,
                            PengBinded::Mutable(PengStated::Initialized(cell)),
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        PengInstruction::PushLocal(local) => match env.get_thread_local(thread_ptr, local).cloned()
        {
            Ok(cell) => match env.push_thread_binded_stated_cell(thread_ptr, cell.clone()) {
                Ok(()) => {}
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        },

        PengInstruction::StoreLocal(local) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(lst) => match env.set_thread_local(thread_ptr, local, lst.clone()) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            }
        }

        PengInstruction::PushHeap(ptr) => {
            let value = match env.get_heap(ptr) {
                Some(coloured) => match coloured.value.value() {
                    PengStated::Initialized(value) => value.clone(),
                    PengStated::Uninitialized => {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                },
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            match value.to_cell() {
                Ok(c) => {
                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(c)),
                    ) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }
                }
                Err(_) => {
                    let ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                        PengStated::Initialized(value),
                    ));
                    match env.push_thread_binded_stated_cell(
                        thread_ptr,
                        PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(ptr))),
                    ) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
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
                Err(e) => return Err(e),
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
                            _ => return Err(PengError::Code(PengErrorCode::TestError)),
                        };

                        let value = match value_cell.value() {
                            PengStated::Initialized(cell) => {
                                match env.get_value_from_cell(cell.clone()) {
                                    Ok(value) => value,
                                    Err(e) => return Err(e),
                                }
                            }
                            PengStated::Uninitialized => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        };

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => match env.assign_heap(ptr, value) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            }
        }

        PengInstruction::CreateEmptyObject => {
            let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                PengStated::Initialized(PengValue::Object(PengObject::new_empty())),
            ));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        PengInstruction::CreateEmptyModule => {
            let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                PengStated::Initialized(PengValue::Module(PengModule::new_empty())),
            ));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        PengInstruction::CreateVector(size) => {
            match env.get_thread_latest_n_binded_stated_cells_cloned(thread_ptr, size) {
                Ok(cells) => match env.pop_thread_stack_n_times(thread_ptr, size) {
                    Ok(()) => {
                        let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                            PengStated::Initialized(PengValue::Vector(PengVector::new(cells))),
                        ));

                        match env.push_thread_binded_stated_cell(
                            thread_ptr,
                            PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                heap_ptr,
                            ))),
                        ) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                    }

                    Err(e) => return Err(e),
                },

                Err(e) => return Err(e),
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
                                        PengValue::Type(PengType::Custom(custom_type)) => {
                                            custom_types.push(custom_type);
                                        }

                                        _ => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    },

                                    Err(e) => return Err(e),
                                }
                            }

                            PengStated::Uninitialized => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        }
                    }

                    match env.pop_thread_stack_n_times(thread_ptr, types_count) {
                        Ok(()) => {
                            let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                                PengStated::Initialized(PengValue::Type(PengType::Custom(
                                    PengCustomType::new_super_type(custom_types),
                                ))),
                            ));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                                Ok(PengValue::Type(PengType::Custom(custom_type))) => custom_type,

                                Ok(_) => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }

                                Err(e) => return Err(e),
                            }
                        }

                        PengStated::Uninitialized => {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                                PengStated::Initialized(PengValue::Object(PengObject::new(
                                    custom_type.fields,
                                ))),
                            ));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
            }
        }

        PengInstruction::Duplicate => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => match env.push_thread_binded_stated_cell(thread_ptr, cell.clone()) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },

                Err(e) => return Err(e),
            }
        }

        PengInstruction::Pop => match env.pop_thread_stack_n_times(thread_ptr, 1) {
            Ok(()) => {}
            Err(e) => return Err(e),
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
                            _ => return Err(PengError::Code(PengErrorCode::TestError)),
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(object_ptr) {
                            Some(coloured) => match &mut coloured.value {
                                PengBinded::Mutable(PengStated::Initialized(
                                    PengValue::Object(object),
                                )) => {
                                    object.fields.insert(name_ptr, value);
                                }

                                _ => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            },

                            None => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                    }

                    Err(e) => return Err(e),
                },

                Err(e) => return Err(e),
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
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    let value = match env.get_heap(object_ptr) {
                        Some(coloured) => match &coloured.value {
                            PengBinded::Mutable(PengStated::Initialized(PengValue::Object(
                                object,
                            ))) => match object.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            },

                            _ => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        },

                        None => {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => match env.push_thread_binded_stated_cell(thread_ptr, value) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        },

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                            _ => return Err(PengError::Code(PengErrorCode::TestError)),
                        };

                        let value = value_cell.clone();

                        match env.get_heap_mut(module_ptr) {
                            Some(coloured) => match &mut coloured.value {
                                PengBinded::Mutable(PengStated::Initialized(
                                    PengValue::Module(module),
                                )) => {
                                    module.members.insert(name_ptr, value);
                                }

                                _ => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            },

                            None => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        }

                        match env.pop_thread_stack_n_times(thread_ptr, 2) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        }
                    }

                    Err(e) => return Err(e),
                },

                Err(e) => return Err(e),
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
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    let value = match env.get_heap(module_ptr) {
                        Some(coloured) => match &coloured.value {
                            PengBinded::Mutable(PengStated::Initialized(PengValue::Module(
                                module,
                            ))) => match module.members.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            },

                            _ => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        },

                        None => {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => match env.push_thread_binded_stated_cell(thread_ptr, value) {
                            Ok(()) => {}
                            Err(e) => return Err(e),
                        },

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
            }
        }

        PengInstruction::Jump(target) => match env.set_thread_program_counter(thread_ptr, target) {
            Ok(()) => {}
            Err(e) => return Err(e),
        },

        PengInstruction::JumpIfTrue(address) => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(condition_cell) => {
                    let condition = match condition_cell.value() {
                        PengStated::Initialized(PengCell::Bool(value)) => *value,
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            if condition {
                                match env.set_thread_program_counter(thread_ptr, address) {
                                    Ok(()) => {}
                                    Err(e) => return Err(e),
                                }
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            if !condition {
                                match env.set_thread_program_counter(thread_ptr, address) {
                                    Ok(()) => {}
                                    Err(e) => return Err(e),
                                }
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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

                Err(e) => return Err(e),
            }
        }

        PengInstruction::FunctionCall(args_count) => match env.get_thread_stack_len(thread_ptr) {
            Ok(stack_len) => {
                if stack_len < args_count + 1 {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }

                let function_index = stack_len - args_count - 1;

                match env.get_thread_latest_binded_stated_cell(thread_ptr, args_count) {
                    Ok(function_cell) => {
                        let function_ptr = match function_cell.value() {
                            PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                            _ => return Err(PengError::Code(PengErrorCode::TestError)),
                        };

                        match env.pop_thread_stack_at(thread_ptr, args_count) {
                            Ok(_) => {
                                match env.push_thread_frame(
                                    thread_ptr,
                                    PengFrame::new(function_ptr, function_index, args_count),
                                ) {
                                    Ok(()) => return Ok(None),
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
        },

        PengInstruction::PushString(name_ptr) => {
            let heap_ptr = env.create_binded_stated_heap(PengBinded::Immutable(
                PengStated::Initialized(PengValue::String(match env.get_pooled_name(name_ptr) {
                    Some(name) => name.clone(),
                    None => {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                })),
            ));

            match env.push_thread_binded_stated_cell(
                thread_ptr,
                PengBinded::Immutable(PengStated::Initialized(PengCell::Reference(heap_ptr))),
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
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
                                        PengValue::Type(value) => {
                                            values.push(value);
                                        }

                                        _ => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    },

                                    Err(e) => return Err(e),
                                }
                            }

                            PengStated::Uninitialized => {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        }
                    }

                    match env.pop_thread_stack_n_times(thread_ptr, count) {
                        Ok(()) => {
                            let heap_ptr = env.create_binded_stated_heap(PengBinded::Mutable(
                                PengStated::Initialized(PengValue::Union(PengUnion {
                                    unions: values,
                                })),
                            ));

                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(PengCell::Reference(
                                    heap_ptr,
                                ))),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let value = match value_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let target_type = match target_value {
                                PengValue::Type(value) => value,
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let converted = match value.convert(target_type) {
                                Ok(value) => value,
                                Err(e) => return Err(e),
                            };

                            let cell = env.get_cell_from_value(converted);

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(cell)),
                                    ) {
                                        Ok(()) => {}
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_add(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_add(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa + bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa + bbb)
                                }
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_sub(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_sub(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa - bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa - bbb)
                                }
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_mul(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    match aaa.checked_mul(*bbb) {
                                        Some(v) => PengCell::Byte(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa * bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa * bbb)
                                }
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_div(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    if *bbb == 0 {
                                        return Err(PengError::Code(PengErrorCode::TestError));
                                    }

                                    PengCell::Byte(*aaa / *bbb)
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa / bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa / bbb)
                                }
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }

                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa.powf(*bbb))
                                }

                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa.powf(*bbb))
                                }

                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
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
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Uint(aaa), PengCell::Uint(bbb)) => {
                                    match aaa.checked_rem(*bbb) {
                                        Some(v) => PengCell::Uint(v),
                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }
                                }
                                (PengCell::Byte(aaa), PengCell::Byte(bbb)) => {
                                    if *bbb == 0 {
                                        return Err(PengError::Code(PengErrorCode::TestError));
                                    }

                                    PengCell::Byte(*aaa % *bbb)
                                }
                                (PengCell::Float32(aaa), PengCell::Float32(bbb)) => {
                                    PengCell::Float32(aaa % bbb)
                                }
                                (PengCell::Float64(aaa), PengCell::Float64(bbb)) => {
                                    PengCell::Float64(aaa % bbb)
                                }
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };
                    match env.pop_thread_stack_n_times(thread_ptr, 2) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(res)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
        PengInstruction::Negate => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        PengStated::Initialized(cell) => {
                            match env.get_value_from_cell(cell.clone()) {
                                Ok(value) => value,
                                Err(e) => return Err(e),
                            }
                        }

                        PengStated::Uninitialized => {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    };

                    let value = match value {
                        PengValue::Int(v) => PengValue::Int(-v),
                        PengValue::Float32(v) => PengValue::Float32(-v),
                        PengValue::Float64(v) => PengValue::Float64(-v),
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    };

                    let cell = env.get_cell_from_value(value);

                    match env.pop_thread_stack_n_times(thread_ptr, 1) {
                        Ok(()) => {
                            match env.push_thread_binded_stated_cell(
                                thread_ptr,
                                PengBinded::Mutable(PengStated::Initialized(cell)),
                            ) {
                                Ok(()) => {}
                                Err(e) => return Err(e),
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let value = match (left, right) {
                                (PengValue::String(a), PengValue::String(b)) => {
                                    PengValue::String(format!("{}{}", a, b))
                                }

                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let cell = env.get_cell_from_value(value);

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(
                                        thread_ptr,
                                        PengBinded::Mutable(PengStated::Initialized(cell)),
                                    ) {
                                        Ok(()) => {}
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
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
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
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(PengCell::Bool(value)) => *value,
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
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

        PengInstruction::Not => {
            match env
                .get_thread_latest_binded_stated_cell(thread_ptr, 0)
                .cloned()
            {
                Ok(cell) => {
                    let value = match cell.value() {
                        PengStated::Initialized(PengCell::Bool(value)) => *value,
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
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
                                Err(e) => return Err(e),
                            }
                        }

                        Err(e) => return Err(e),
                    }
                }

                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
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

        PengInstruction::TryFunctionCall(args_count) => {
            match env.execute_try_function_call(thread_ptr, args_count) {
                Ok(()) => return Ok(None),
                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let result = match left.greater_than(&right) {
                                Ok(result) => result,
                                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let result = match left.greater_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let result = match left.less_than(&right) {
                                Ok(result) => result,
                                Err(e) => return Err(e),
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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let left = match left_cell.value() {
                                PengStated::Initialized(cell) => {
                                    match env.get_value_from_cell(cell.clone()) {
                                        Ok(value) => value,
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let result = match left.less_equals_than(&right) {
                                Ok(result) => result,
                                Err(e) => return Err(e),
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
                                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                                    };

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
                                                                    PengFrame::new(
                                                                        operation_ptr,
                                                                        base,
                                                                        2,
                                                                    ),
                                                                ) {
                                                                    Ok(()) => return Ok(None),
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
                                        },

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
                                        Err(e) => return Err(e),
                                    }
                                }

                                PengStated::Uninitialized => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            let index = match index_value {
                                PengValue::Int(v) => {
                                    if v < 0 {
                                        return Err(PengError::Code(PengErrorCode::TestError));
                                    }

                                    v as usize
                                }

                                PengValue::Uint(v) => v,

                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let object_ptr = match object_cell.value() {
                                PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                                _ => return Err(PengError::Code(PengErrorCode::TestError)),
                            };

                            let value = match env.get_heap(object_ptr) {
                                Some(coloured) => match coloured.value.value() {
                                    PengStated::Initialized(PengValue::Vector(vector)) => {
                                        match vector.values.get(index) {
                                            Some(value) => value.clone(),
                                            None => {
                                                return Err(PengError::Code(
                                                    PengErrorCode::TestError,
                                                ));
                                            }
                                        }
                                    }

                                    _ => {
                                        return Err(PengError::Code(PengErrorCode::TestError));
                                    }
                                },

                                None => {
                                    return Err(PengError::Code(PengErrorCode::TestError));
                                }
                            };

                            match env.pop_thread_stack_n_times(thread_ptr, 2) {
                                Ok(()) => {
                                    match env.push_thread_binded_stated_cell(thread_ptr, value) {
                                        Ok(()) => {}
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
                                                Err(e) => return Err(e),
                                            }
                                        }

                                        PengStated::Uninitialized => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    };

                                    let index = match index_value {
                                        PengValue::Int(v) => {
                                            if v < 0 {
                                                return Err(PengError::Code(
                                                    PengErrorCode::TestError,
                                                ));
                                            }

                                            v as usize
                                        }

                                        PengValue::Uint(v) => v,

                                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                                    };

                                    let object_ptr = match object_cell.value() {
                                        PengStated::Initialized(PengCell::Reference(ptr)) => *ptr,
                                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                                    };

                                    match env.get_heap_mut(object_ptr) {
                                        Some(coloured) => match &mut coloured.value {
                                            PengBinded::Mutable(PengStated::Initialized(
                                                PengValue::Vector(vector),
                                            )) => {
                                                if index >= vector.values.len() {
                                                    return Err(PengError::Code(
                                                        PengErrorCode::TestError,
                                                    ));
                                                }

                                                vector.values[index] = value_cell;
                                            }

                                            _ => {
                                                return Err(PengError::Code(
                                                    PengErrorCode::TestError,
                                                ));
                                            }
                                        },

                                        None => {
                                            return Err(PengError::Code(PengErrorCode::TestError));
                                        }
                                    }

                                    match env.pop_thread_stack_n_times(thread_ptr, 3) {
                                        Ok(()) => {}
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
    }
    Ok(None)
}
