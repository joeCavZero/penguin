use crate::core::*;

pub fn step_thread(
    env: &mut PengEnv,
    thread: &mut PengThread,
) -> Result<bool, PengError> {
    let mut frame = match thread.frames.pop() {
        Some(frame) => frame,
        None => return Ok(true),
    };

    let finished = match step_frame(&mut frame, env, thread) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if !finished {
        thread.frames.push(frame);
    }

    Ok(thread.frames.is_empty())
}


pub fn step_frame(
    frame: &mut PengFrame,
    env: &mut PengEnv,
    thread: &mut PengThread,
) -> Result<bool, PengError> {
    match frame {
        PengFrame::Bytecode(btc_frame) => {
            let (instr, constant): (PengInstruction, PengValue) = match env.get_value(btc_frame.function) {
                Some(val) => {
                    if let PengValue::Function(func) = val {
                        match func {
                            PengFunction::Bytecode(btc_func) => {
                                match btc_func.bytecode.get(btc_frame.instruction_counter) {
                                    Some(instr) => match instr.clone() {
                                        PengInstruction::PushConst(c) => {
                                            (
                                                instr.clone(),
                                                match btc_func.consts.get(c) {
                                                    Some(v) =>  v.clone(),
                                                    None => return Err(PengError::Code(PengErrorCode::TestError)),
                                                }
                                            )
                                        }
                                        _ => {
                                            (instr.clone(), PengValue::Nil)
                                        }
                                    },
                                    None => return Ok(true),
                                }
                            }
                            PengFunction::Native(ntv_func) => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                None => todo!(),
            };
            return execute_instruction(instr, constant, btc_frame, env, thread);
        }
        PengFrame::Native(_ntv) => {
            todo!();
        }
    }
}


pub fn execute_instruction(
    instruction: PengInstruction,
    constant: PengValue,
    btc_frame: &mut PengBytecodeFrame,
    env: &mut PengEnv,
    thread: &mut PengThread,
) -> Result<bool, PengError> {
    btc_frame.instruction_counter += 1;

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
                _ => PengCell::Reference(
                    env.create_value(PengBinded::Mutable(constant) )
                )
            };

            thread.stack.push(cell);
            Ok(false)
        }

        PengInstruction::PushLocal(a) => {
            let ptr = match thread.stack.get(btc_frame.base.saturating_add(a)) {
                Some(v) => v.clone(),
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };
            thread.stack.push(ptr);
            Ok(false)
        }

        PengInstruction::StoreLocal(a) => {
            let value = match thread.stack.pop() {
                Some(v) => v,
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };
            match thread.stack.get_mut(btc_frame.base.saturating_add(a)) {
                Some(v) => *v = value,
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            }
            Ok(false)
        }

        PengInstruction::PushValue(a) => {
            let cell = match env.get_value(a) {
                Some(v) => {
                    match v {
                        PengValue::Nil => PengCell::Nil,
                        PengValue::Int(aux) => PengCell::Int(*aux),
                        PengValue::Uint(aux) => PengCell::Uint(*aux),
                        PengValue::Float32(aux) => PengCell::Float32(*aux),
                        PengValue::Float64(aux) => PengCell::Float64(*aux),
                        PengValue::Byte(aux) => PengCell::Byte(*aux),
                        PengValue::Bool(aux) => PengCell::Bool(*aux),
                        _ => PengCell::Reference(a),
                    }
                }
                None => return Err(PengError::Code(PengErrorCode::TestError)),
            };

            thread.stack.push(cell);
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

                PengCell::Reference(value_ptr) => {
                    match env.get_value(value_ptr) {
                        Some(v) => v.clone(),
                        None => return Err(PengError::Code(PengErrorCode::TestError)),
                    }
                }
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

            let ptr = env.create_value(
                PengBinded::Mutable(PengValue::String(string))
            );

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
            let value = thread.stack.last()
                .ok_or_else(|| PengError::Message("stack underflow".to_string()))?
                .clone();

            thread.stack.push(value);
            Ok(false)
        }

        PengInstruction::Pop => {
            thread.stack.pop()
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
            btc_frame.instruction_counter = a;
            Ok(false)
        }

        PengInstruction::JumpIfTrue(a) => {
            todo!()
        }

        PengInstruction::JumpIfFalse(a) => {
            todo!()
        }

        PengInstruction::Return => {
            Ok(true)
        }
    }
}