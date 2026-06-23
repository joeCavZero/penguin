use crate::core::*;

/*
pub fn step_thread(env: &mut PengEnv, thread_ptr: PengHeapPtr) -> Result<bool, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, _frame_params_count) =
        match env.get_heap(thread_ptr) {
            Some(PengBinded::Immutable(tval)) | Some(PengBinded::Mutable(tval)) => {
                if let PengValue::Thread(thread) = tval {
                    if let Some(last_frame) = thread.frames.last() {
                        match last_frame {
                            PengFrame::Bytecode(fr_btc) => (
                                fr_btc.program_counter,
                                fr_btc.base,
                                fr_btc.function,
                                fr_btc.params_count,
                            ),
                            PengFrame::Native(_fr_ntv) => {
                                todo!()
                            }
                        }
                    } else {
                        return Ok(true);
                    }
                } else {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }
            }

            Some(PengBinded::UninitializedImmutable) => {
                return Err(PengError::Code(PengErrorCode::TestError));
            }

            None => {
                return Err(PengError::Code(PengErrorCode::TestError));
            }
        };

    let (should_end_frame, instruction, constant, _generics_count) =
        match env.get_heap(frame_function_ptr) {
            Some(PengBinded::Immutable(fval)) | Some(PengBinded::Mutable(fval)) => {
                if let PengValue::Function(func) = fval {
                    match func {
                        PengFunction::Bytecode(func_btc) => {
                            let instr = match func_btc.bytecode.get(frame_program_counter) {
                                Some(i) => i.clone(),
                                None => {
                                    return Ok(true);
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
                                func_btc.generics_count,
                            )
                        }

                        PengFunction::Native(_func_ntv) => {
                            todo!()
                        }
                    }
                } else {
                    return Err(PengError::Code(PengErrorCode::TestError));
                }
            }

            Some(PengBinded::UninitializedImmutable) => {
                return Err(PengError::Code(PengErrorCode::TestError));
            }

            None => {
                return Err(PengError::Code(PengErrorCode::TestError));
            }
        };

    if should_end_frame {
        return Ok(true);
    }

    execute_instruction(instruction, constant, thread_ptr, frame_base, env)?;

    match env.get_heap_mut(thread_ptr) {
        Some(PengBinded::Immutable(_)) => {
            return Err(PengError::Code(PengErrorCode::TestError));
        }

        Some(PengBinded::Mutable(tval)) => {
            if let PengValue::Thread(thread) = tval {
                if let Some(last_frame) = thread.frames.last_mut() {
                    match last_frame {
                        PengFrame::Bytecode(fr_btc) => {
                            fr_btc.program_counter = fr_btc
                                .program_counter
                                .checked_add(1)
                                .ok_or(PengError::Code(PengErrorCode::TestError))?;
                        }

                        PengFrame::Native(_fr_ntv) => {
                            todo!();
                        }
                    }
                } else {
                    return Ok(true);
                }
            } else {
                return Err(PengError::Code(PengErrorCode::TestError));
            }
        }

        Some(PengBinded::UninitializedImmutable) => {
            return Err(PengError::Code(PengErrorCode::TestError));
        }

        None => {
            return Err(PengError::Code(PengErrorCode::TestError));
        }
    }

    Ok(false)
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    actual_thread_ptr: PengHeapPtr,
    frame_base: usize,
    env: &mut PengEnv,
) -> Result<bool, PengError> {
    todo!();
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    actual_thread_ptr: PengValuePtr,
    frame_base: usize,
    env: &mut PengEnv,
) -> Result<bool, PengError> {
    match instruction {
        PengInstruction::PushConst(_) => {
            let cell = match constant {
                PengValue::Nil => PengCell::Nil,
                PengValue::Int(aux) => PengCell::Int(aux),
                PengValue::Uint(aux) => PengCell::Uint(aux),
                PengValue::Float32(aux) => PengCell::Float32(aux),
                PengValue::Float64(aux) => PengCell::Float64(aux),
                PengValue::Byte(aux) => PengCell::Byte(aux),
                PengValue::Bool(aux) => PengCell::Bool(aux),
                _ => PengCell::Reference(env.create_value(PengBinded::Mutable(constant))),
            };

            if let Some(tval) = env.get_value_mut(actual_thread_ptr) {
                match tval {
                    PengBinded::Immutable(value_thread)
                    | PengBinded::Mutable(value_thread) => {
                        if let PengValue::Thread(thread) = value_thread {
                            thread.stack.push(PengBinded::Mutable(cell));
                        }
                    }
                    PengBinded::UninitializedImmutable => return Err(PengError::Code(PengErrorCode::TestError))
                }
                
            } else {
                return Err(PengError::Code(PengErrorCode::TestError));
            }
            Ok(false)
        }

        PengInstruction::PushLocal(a) => {
            match env.get_value(actual_thread_ptr) {
                Some(tval) => {
                    if let PengValue::Thread(thread) = tval {
                        let cell = match thread.stack.get(frame_base.saturating_add(a)) {
                            Some(v) => v.clone(),
                            None => return Err(PengError::Code(PengErrorCode::TestError)),
                        };
                        thread.stack.push(cell);
                    } else {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            }
            Ok(false)
        }

        PengInstruction::StoreLocal(a) => {
            let value = match env.get_value_mut(actual_thread_ptr) {
                Some(tval) => {
                    if let PengValue::Thread(thread) = tval {
                        match thread.stack.pop() {
                            Some(v) => v,
                            None => return Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    } else {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            match env.get_value_mut(actual_thread_ptr) {
                Some(tval) => {
                    if let PengValue::Thread(thread) = tval {
                        match thread.stack.get_mut(frame_base.saturating_add(a)) {
                            Some(local) => {
                                *local = value;
                            }
                            None => return Err(PengError::Code(PengErrorCode::TestError)),
                        }
                    } else {
                        return Err(PengError::Code(PengErrorCode::TestError));
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            }

            Ok(false)
        }

        PengInstruction::PushValue(a) => {
            let cell = match env.get_value(a) {
                Some(v) => match v {
                    PengValue::Nil => PengCell::Nil,
                    PengValue::Int(aux) => PengCell::Int(*aux),
                    PengValue::Uint(aux) => PengCell::Uint(*aux),
                    PengValue::Float32(aux) => PengCell::Float32(*aux),
                    PengValue::Float64(aux) => PengCell::Float64(*aux),
                    PengValue::Byte(aux) => PengCell::Byte(*aux),
                    PengValue::Bool(aux) => PengCell::Bool(*aux),
                    _ => PengCell::Reference(a),
                },
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            thread.stack.push(PengBinded::Mutable(cell));
            Ok(false)
        }

        PengInstruction::PushValueRef(a) => {
            thread.stack.push(PengCell::Reference(a));
            Ok(false)
        }

        PengInstruction::StoreValue => {
            let value = match thread.stack.pop() {
                Some(v) => v,
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            let reference = match thread.stack.pop() {
                Some(v) => v,
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            let ptr = match reference {
                PengCell::Reference(ptr) => ptr,
                _ => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            let value = match value {
                PengCell::Nil => PengValue::Nil,
                PengCell::Int(v) => PengValue::Int(v),
                PengCell::Uint(v) => PengValue::Uint(v),
                PengCell::Float32(v) => PengValue::Float32(v),
                PengCell::Float64(v) => PengValue::Float64(v),
                PengCell::Byte(v) => PengValue::Byte(v),
                PengCell::Bool(v) => PengValue::Bool(v),

                PengCell::Reference(value_ptr) => match env.get_value(value_ptr) {
                    Some(v) => v.clone(),
                    None => return Err(PengError::Code(PengErrorCode::TestError)),
                },
            };

            match env.set_value(ptr, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
            Ok(false)
        }

        PengInstruction::PushString(a) => {
            let string = match env.get_name(a) {
                Some(v) => v.clone(),
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            let ptr = env.create_value(PengBinded::Mutable(PengValue::String(string)));

            thread.stack.push(PengCell::Reference(ptr));
            Ok(false)
        }

        PengInstruction::CreateObjectType => {
            todo!("CreateObjectType");
        }

        PengInstruction::CreateSuperType(_) => {
            todo!("CreateSuperType usando a como quantidade de supers");
        }

        PengInstruction::CreateUnion(_) => {
            todo!("CreateUnion usando a como quantidade de tipos");
        }

        PengInstruction::Convert => {
            todo!("Convert");
        }

        PengInstruction::CheckType => {
            todo!("CheckType");
        }

        PengInstruction::Duplicate => {
            let value = thread
                .stack
                .last()
                .ok_or_else(|| PengError::Message("stack underflow".to_string()))?
                .clone();

            thread.stack.push(value);
            Ok(false)
        }

        PengInstruction::Pop => {
            thread
                .stack
                .pop()
                .ok_or_else(|| PengError::Message("stack underflow".to_string()))?;

            Ok(false)
        }

        PengInstruction::Add => {
            todo!("Add");
        }

        PengInstruction::Subtract => {
            todo!("Subtract");
        }

        PengInstruction::Multiply => {
            todo!("Multiply");
        }

        PengInstruction::Divide => {
            todo!("Divide");
        }

        PengInstruction::Power => {
            todo!("Power");
        }

        PengInstruction::Remainder => {
            todo!("Remainder");
        }

        PengInstruction::Negate => {
            todo!("Negate");
        }

        PengInstruction::Concat => {
            todo!("Concat");
        }

        PengInstruction::And => {
            todo!("And");
        }

        PengInstruction::Or => {
            todo!("Or");
        }

        PengInstruction::Not => {
            todo!("Not");
        }

        PengInstruction::Equals => {
            todo!("Equals");
        }

        PengInstruction::NotEquals => {
            todo!("NotEquals");
        }

        PengInstruction::GreaterThan => {
            todo!("GreaterThan");
        }

        PengInstruction::GreaterEqualsThan => {
            todo!("GreaterEqualsThan");
        }

        PengInstruction::LessThan => {
            todo!("LessThan");
        }

        PengInstruction::LessEqualsThan => {
            todo!("LessEqualsThan");
        }

        PengInstruction::OperationCall => {
            todo!("OperationCall");
        }

        PengInstruction::FunctionCall { .. } => {
            todo!("FunctionCall usando a como generics e b como params");
        }

        PengInstruction::TryFunctionCall { .. } => {
            todo!("TryFunctionCall usando a como generics e b como params");
        }

        PengInstruction::GetIndex => {
            todo!("GetIndex");
        }

        PengInstruction::GetIndexRef => {
            todo!("GetIndexRef");
        }

        PengInstruction::GetConstAttribute(_) => {
            todo!("GetConstAttribute usando a como nome");
        }

        PengInstruction::GetConstAttributeRef(_) => {
            todo!("GetConstAttributeRef usando a como nome");
        }

        PengInstruction::GetConstMember(_) => {
            todo!("GetConstMember usando a como nome");
        }

        PengInstruction::GetConstMemberRef(_) => {
            todo!("GetConstMemberRef usando a como nome");
        }

        PengInstruction::Jump(a) => {
            btc_frame.program_counter = a;
            Ok(false)
        }

        PengInstruction::JumpIfTrue(a) => {
            todo!()
        }

        PengInstruction::JumpIfFalse(a) => {
            todo!()
        }

        PengInstruction::Return => Ok(true),
    }
}
*/