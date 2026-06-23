use crate::core::*;

pub fn step_thread(env: &mut PengEnv, thread_ptr: PengHeapPtr) -> Result<bool, PengError> {
    let (frame_program_counter, frame_base, frame_function_ptr, _frame_params_count) =
        match env.get_heap(thread_ptr) {
            Some(coloured) => {
                if let PengStated::Initialized(tval)= coloured.value.value() {
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
        return Ok(true);
    }

    match execute_instruction(instruction, constant, thread_ptr, frame_base, env)  {
        Ok(_) => {}
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

    Ok(false)
}

pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    actual_thread_ptr: PengHeapPtr,
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
                _ => PengCell::Reference(env.create_binded_stated_heap(PengBinded::Mutable(PengStated::Initialized(constant)))),
            };
            todo!()
        }

        PengInstruction::PushLocal(_) => {
            todo!()
        }

        PengInstruction::StoreLocal(_) => {
            todo!()
        }

        PengInstruction::PushHeap(_) => {
            todo!()
        }

        PengInstruction::PushHeapRef(_) => {
            todo!()
        }

        PengInstruction::StoreHeap => {
            todo!()
        }

        PengInstruction::PushString(_) => {
            todo!()
        }

        PengInstruction::CreateEmptyModule => {
            todo!()
        }

        PengInstruction::CreateEmptyObject => {
            todo!()
        }

        PengInstruction::CreateVector(_) => {
            todo!()
        }

        PengInstruction::CreateTypedObject => {
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
            todo!()
        }

        PengInstruction::Pop => {
            todo!()
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

        PengInstruction::FunctionCall (_) => {
            todo!();
        }

        PengInstruction::TryFunctionCall (_) => {
            todo!();
        }

        PengInstruction::GetIndex => {
            todo!("GetIndex");
        }

        PengInstruction::SetIndex => {
            todo!("GetIndexRef");
        }

        PengInstruction::GetConstAttribute(_) => {
            todo!("GetConstAttribute usando a como nome");
        }

        PengInstruction::SetConstAttribute(_) => {
            todo!("GetConstAttributeRef usando a como nome");
        }

        PengInstruction::GetConstMember(_) => {
            todo!("GetConstMember usando a como nome");
        }

        PengInstruction::SetConstMember(_) => {
            todo!("GetConstMemberRef usando a como nome");
        }

        PengInstruction::Jump(_) => {
            todo!()
        }

        PengInstruction::JumpIfTrue(_) => {
            todo!()
        }

        PengInstruction::JumpIfFalse(_) => {
            todo!()
        }

        PengInstruction::Return => {
            todo!()
        }
    }
}