use std::collections::HashMap;

use crate::core::*;

pub fn step_thread(env: &mut PengEnv, thread_ptr: PengHeapPtr) -> Result<Option<PengBindedStatedCell>, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, _frame_params_count) =
        match env.get_heap(thread_ptr) {
            Some(coloured) => {
                if let PengStated::Initialized(tval)= coloured.value.value() {
                    if let PengValue::Thread(thread) = tval {
                        if let Some(last_frame) = thread.frames.last() {
                            (
                                last_frame.program_counter,
                                last_frame.base,
                                last_frame.function,
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
            None => todo!()
        };

    let (should_end_frame, instruction, constant) =
        match env.get_heap(frame_function_ptr) {
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

                                (
                                    false,
                                    instr,
                                    constant,
                                )
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

    match execute_instruction(instruction, constant, thread_ptr, frame_base, env)  {
        Ok(res) => {
            match res {
                Some(ret) => return Ok(Some(ret)),
                None => {}
            }
        }
        Err(e) => return Err(e),
    };

    match env.get_heap_mut(thread_ptr) {
        Some(coloured) => {
            match &mut coloured.value {
                PengBinded::Immutable(_) => {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }

                PengBinded::Mutable(mutval) => {
                    match mutval {
                        PengStated::Initialized(tval) => {
                            if let PengValue::Thread(thread) = tval {
                                if let Some(last_frame) = thread.frames.last_mut() {
                                    last_frame.program_counter = match last_frame.program_counter.checked_add(1) {
                                        Some(r) => r,
                                        None => return Err(PengError::Code(PengErrorCode::TestError))
                                    };
                                } else {
                                    return Ok(None);
                                }
                            } else {
                                return Err(PengError::Code(PengErrorCode::TestError));
                            }
                        }
                        PengStated::Uninitialized => {
                            todo!();
                        }
                    }
                }
            }
        } 
        None => {
            todo!()
        }
    }

    Ok(None)
}
pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    actual_thread_ptr: PengHeapPtr,
    frame_base: usize,
    env: &mut PengEnv,
) -> Result<Option<PengBindedStatedCell>, PengError> {
    fn initialized_cell(cell: PengCell) -> PengBindedStatedCell {
        PengBinded::Mutable(PengStated::Initialized(cell))
    }

    fn nil_cell() -> PengBindedStatedCell {
        initialized_cell(PengCell::Nil)
    }

    fn value_to_cell(
        value: PengValue,
        env: &mut PengEnv,
    ) -> PengBindedStatedCell {
        match value {
            PengValue::Nil => initialized_cell(PengCell::Nil),
            PengValue::Int(v) => initialized_cell(PengCell::Int(v)),
            PengValue::Uint(v) => initialized_cell(PengCell::Uint(v)),
            PengValue::Float32(v) => initialized_cell(PengCell::Float32(v)),
            PengValue::Float64(v) => initialized_cell(PengCell::Float64(v)),
            PengValue::Byte(v) => initialized_cell(PengCell::Byte(v)),
            PengValue::Bool(v) => initialized_cell(PengCell::Bool(v)),
            other => {
                let ptr = env.create_binded_stated_heap(
                    PengBinded::Mutable(PengStated::Initialized(other))
                );

                initialized_cell(PengCell::Reference(ptr))
            }
        }
    }

    fn cell_to_value(
        cell: &PengBindedStatedCell,
        env: &PengEnv,
    ) -> Result<PengValue, PengError> {
        match cell.value() {
            PengStated::Initialized(cell_value) => {
                match cell_value {
                    PengCell::Nil => Ok(PengValue::Nil),
                    PengCell::Int(v) => Ok(PengValue::Int(*v)),
                    PengCell::Uint(v) => Ok(PengValue::Uint(*v)),
                    PengCell::Float32(v) => Ok(PengValue::Float32(*v)),
                    PengCell::Float64(v) => Ok(PengValue::Float64(*v)),
                    PengCell::Byte(v) => Ok(PengValue::Byte(*v)),
                    PengCell::Bool(v) => Ok(PengValue::Bool(*v)),
                    PengCell::Reference(ptr) => {
                        match env.get_heap(*ptr) {
                            Some(coloured) => {
                                match coloured.value.value() {
                                    PengStated::Initialized(value) => Ok(value.clone()),
                                    PengStated::Uninitialized => {
                                        Err(PengError::Code(PengErrorCode::TestError))
                                    }
                                }
                            }
                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    }
                }
            }
            PengStated::Uninitialized => {
                Err(PengError::Code(PengErrorCode::TestError))
            }
        }
    }

    fn pop_stack(
        env: &mut PengEnv,
        thread_ptr: PengHeapPtr,
    ) -> Result<PengBindedStatedCell, PengError> {
        match env.get_heap_mut(thread_ptr) {
            Some(coloured) => {
                match &mut coloured.value {
                    PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                        match thread.stack.pop() {
                            Some(value) => Ok(value),
                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    }
                    _ => Err(PengError::Code(PengErrorCode::TestError)),
                }
            }
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    fn push_stack(
        env: &mut PengEnv,
        thread_ptr: PengHeapPtr,
        value: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match env.get_heap_mut(thread_ptr) {
            Some(coloured) => {
                match &mut coloured.value {
                    PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                        thread.stack.push(value);
                        Ok(())
                    }
                    _ => Err(PengError::Code(PengErrorCode::TestError)),
                }
            }
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    fn get_local(
        env: &mut PengEnv,
        thread_ptr: PengHeapPtr,
        frame_base: usize,
        local: usize,
    ) -> Result<PengBindedStatedCell, PengError> {
        match env.get_heap(thread_ptr) {
            Some(coloured) => {
                match coloured.value.value() {
                    PengStated::Initialized(PengValue::Thread(thread)) => {
                        match thread.stack.get(frame_base + local) {
                            Some(value) => Ok(value.clone()),
                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    }
                    _ => Err(PengError::Code(PengErrorCode::TestError)),
                }
            }
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    fn set_local(
        env: &mut PengEnv,
        thread_ptr: PengHeapPtr,
        frame_base: usize,
        local: usize,
        value: PengBindedStatedCell,
    ) -> Result<(), PengError> {
        match env.get_heap_mut(thread_ptr) {
            Some(coloured) => {
                match &mut coloured.value {
                    PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                        let index = frame_base + local;

                        while thread.stack.len() < index {
                            thread.stack.push(nil_cell());
                        }

                        if thread.stack.len() == index {
                            thread.stack.push(value);
                        } else {
                            thread.stack[index] = value;
                        }

                        Ok(())
                    }
                    _ => Err(PengError::Code(PengErrorCode::TestError)),
                }
            }
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    fn reference_ptr_from_cell(
        cell: &PengBindedStatedCell,
    ) -> Result<PengHeapPtr, PengError> {
        match cell.value() {
            PengStated::Initialized(PengCell::Reference(ptr)) => Ok(*ptr),
            _ => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    fn bool_from_cell(
        cell: &PengBindedStatedCell,
        env: &PengEnv,
    ) -> Result<bool, PengError> {
        let value = match cell_to_value(cell, env) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        match value {
            PengValue::Bool(v) => Ok(v),
            PengValue::Nil => Ok(false),
            _ => Ok(true),
        }
    }

    fn set_program_counter(
        env: &mut PengEnv,
        thread_ptr: PengHeapPtr,
        target: usize,
    ) -> Result<(), PengError> {
        match env.get_heap_mut(thread_ptr) {
            Some(coloured) => {
                match &mut coloured.value {
                    PengBinded::Mutable(PengStated::Initialized(PengValue::Thread(thread))) => {
                        match thread.frames.last_mut() {
                            Some(frame) => {
                                if target == 0 {
                                    frame.program_counter = 0;
                                } else {
                                    frame.program_counter = target - 1;
                                }

                                Ok(())
                            }
                            None => Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    }
                    _ => Err(PengError::Code(PengErrorCode::TestError)),
                }
            }
            None => Err(PengError::Code(PengErrorCode::TestError)),
        }
    }

    match instruction {
        PengInstruction::PushConst(_) => {
            let cell = value_to_cell(constant, env);
            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::PushLocal(local) => {
            let value = match get_local(env, actual_thread_ptr, frame_base, local) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            match push_stack(env, actual_thread_ptr, value) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::StoreLocal(local) => {
            let value = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            match set_local(env, actual_thread_ptr, frame_base, local, value) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::PushHeap(ptr) => {
            let value = match env.get_heap(ptr) {
                Some(coloured) => {
                    match coloured.value.value() {
                        PengStated::Initialized(value) => value.clone(),
                        PengStated::Uninitialized => {
                            return Err(PengError::Code(PengErrorCode::TestError));
                        }
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::PushHeapRef(ptr) => {
            let cell = initialized_cell(PengCell::Reference(ptr));

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::StoreHeap => {
            let value_cell = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ref_cell = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ptr = match reference_ptr_from_cell(&ref_cell) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };

            let value = match cell_to_value(&value_cell, env) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            match env.assign_heap(ptr, value) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::CreateEmptyObject => {
            let value = PengValue::Object(PengObject {
                fields: HashMap::new(),
            });

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::CreateEmptyModule => {
            let value = PengValue::Module(PengModule {
                members: HashMap::new(),
            });

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::CreateVector(count) => {
            let mut values = Vec::new();
            let mut index = 0usize;

            while index < count {
                let value = match pop_stack(env, actual_thread_ptr) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                values.push(value);
                index += 1;
            }

            values.reverse();

            let value = PengValue::Vector(PengVector {
                values,
            });

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::CreateSuperType(count) => {
            let mut fields = HashMap::new();
            let mut index = 0usize;

            while index < count {
                let super_cell = match pop_stack(env, actual_thread_ptr) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                let super_value = match cell_to_value(&super_cell, env) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                match super_value {
                    PengValue::Type(PengType::Custom(custom)) => {
                        for (name, field) in custom.fields {
                            fields.insert(name, field);
                        }
                    }
                    _ => return Err(PengError::Code(PengErrorCode::TestError)),
                }

                index += 1;
            }

            let value = PengValue::Type(PengType::Custom(PengCustomType {
                fields,
            }));

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::CreateTypedObject => {
            let type_cell = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let type_value = match cell_to_value(&type_cell, env) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let mut fields = HashMap::new();

            match type_value {
                PengValue::Type(PengType::Custom(custom)) => {
                    for (name, field) in custom.fields {
                        fields.insert(name, field);
                    }
                }
                _ => return Err(PengError::Code(PengErrorCode::TestError)),
            }

            let value = PengValue::Object(PengObject {
                fields,
            });

            let cell = value_to_cell(value, env);

            match push_stack(env, actual_thread_ptr, cell) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::Duplicate => {
            let value = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            match push_stack(env, actual_thread_ptr, value.clone()) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match push_stack(env, actual_thread_ptr, value) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::Pop => {
            match pop_stack(env, actual_thread_ptr) {
                Ok(_) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::SetConstAttribute(name) => {
            let value = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let object = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ptr = match reference_ptr_from_cell(&object) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };

            match env.get_heap_mut(ptr) {
                Some(coloured) => {
                    match &mut coloured.value {
                        PengBinded::Mutable(PengStated::Initialized(PengValue::Object(object))) => {
                            object.fields.insert(name, value);
                            Ok(None)
                        }
                        PengBinded::Mutable(PengStated::Initialized(PengValue::Type(PengType::Custom(custom)))) => {
                            custom.fields.insert(name, value);
                            Ok(None)
                        }
                        _ => Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }
                None => Err(PengError::Code(PengErrorCode::TestError)),
            }
        }

        PengInstruction::GetConstAttribute(name) => {
            let object = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ptr = match reference_ptr_from_cell(&object) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };

            let field = match env.get_heap(ptr) {
                Some(coloured) => {
                    match coloured.value.value() {
                        PengStated::Initialized(PengValue::Object(object)) => {
                            match object.fields.get(&name) {
                                Some(value) => value.clone(),
                                None => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        PengStated::Initialized(PengValue::Type(PengType::Custom(custom))) => {
                            match custom.fields.get(&name) {
                                Some(value) => value.clone(),
                                None => return Err(PengError::Code(PengErrorCode::TestError)),
                            }
                        }
                        _ => return Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            match push_stack(env, actual_thread_ptr, field) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::SetConstMember(name_ptr) => {
            let value = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let module = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ptr = match reference_ptr_from_cell(&module) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };

            match env.get_heap_mut(ptr) {
                Some(coloured) => {
                    match &mut coloured.value {
                        PengBinded::Mutable(PengStated::Initialized(PengValue::Module(module))) => {
                            module.members.insert(name_ptr, value);
                            Ok(None)
                        }
                        _ => Err(PengError::new_message(
                            "expected module".to_string(),
                        )),
                    }
                }
                None => Err(PengError::new_message(
                    "module not found".to_string(),
                )),
            }
        }

        PengInstruction::GetConstMember(name_ptr) => {
            let module = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let ptr = match reference_ptr_from_cell(&module) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };

            let member = match env.get_heap(ptr) {
                Some(coloured) => {
                    match &coloured.value {
                        PengBinded::Mutable(PengStated::Initialized(PengValue::Module(module)))
                        | PengBinded::Immutable(PengStated::Initialized(PengValue::Module(module))) => {
                            match module.members.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => {
                                    return Err(PengError::new_message(
                                        "module member not found".to_string(),
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(PengError::new_message(
                                "expected module".to_string(),
                            ));
                        }
                    }
                }
                None => {
                    return Err(PengError::new_message(
                        "module not found".to_string(),
                    ));
                }
            };

            match push_stack(env, actual_thread_ptr, member) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::Return => {
            let value = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            Ok(Some(value))
        }

        PengInstruction::Jump(target) => {
            match set_program_counter(env, actual_thread_ptr, target) {
                Ok(()) => Ok(None),
                Err(e) => Err(e),
            }
        }

        PengInstruction::JumpIfTrue(target) => {
            let condition = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let should_jump = match bool_from_cell(&condition, env) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            if should_jump {
                match set_program_counter(env, actual_thread_ptr, target) {
                    Ok(()) => Ok(None),
                    Err(e) => Err(e),
                }
            } else {
                Ok(None)
            }
        }

        PengInstruction::JumpIfFalse(target) => {
            let condition = match pop_stack(env, actual_thread_ptr) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let should_jump = match bool_from_cell(&condition, env) {
                Ok(value) => !value,
                Err(e) => return Err(e),
            };

            if should_jump {
                match set_program_counter(env, actual_thread_ptr, target) {
                    Ok(()) => Ok(None),
                    Err(e) => Err(e),
                }
            } else {
                Ok(None)
            }
        }

        _ => Err(PengError::Code(PengErrorCode::TestError)),
    }
}