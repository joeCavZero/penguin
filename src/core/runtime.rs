use crate::core::*;

pub fn step_thread(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    unit: &PengUnit,
) -> Result<Option<PengBindedCell>, PengError> {
    let (
        frame_program_counter,
        frame_base,
        frame_function_ptr,
        frame_params_count,
        frame_call_position,
    ) = match env.get_heap(thread) {
        Some(PengValue::Box(PengBox::Thread(thread))) => {
            if let Some(last_frame) = thread.frames.last() {
                (
                    last_frame.program_counter,
                    last_frame.base,
                    last_frame.procedure,
                    last_frame.params_count,
                    last_frame.call_position.clone(),
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

    let (ntv_opt, should_end_frame, instruction, instruction_position, constant): (
        Option<PengNativeCallable>,
        bool,
        PengInstruction,
        PengPosition,
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

                let instr_pos = match func_btc.positions.get(frame_program_counter) {
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

                (None, false, instr, instr_pos, constant)
            }

            PengFunction::Native(func_ntv) => {
                let position = match frame_call_position.clone() {
                    Some(position) => position,
                    None => PengPosition::new(0, 0, None),
                };

                (
                    Some(PengNativeCallable::Function(func_ntv.clone())),
                    false,
                    PengInstruction::FunctionCall(frame_params_count),
                    position,
                    None,
                )
            }
        },
        Some(PengValue::Box(PengBox::Operation(operation))) => match operation {
            PengOperation::Bytecode(operation_btc) => {
                let instr = match operation_btc.bytecode.get(frame_program_counter) {
                    Some(i) => i.clone(),
                    None => {
                        return Ok(None);
                    }
                };

                let instr_pos = match operation_btc.positions.get(frame_program_counter) {
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

                (None, false, instr, instr_pos, constant)
            }

            PengOperation::Native(operation_ntv) => {
                let position = match frame_call_position.clone() {
                    Some(position) => position,
                    None => PengPosition::new(0, 0, None),
                };

                (
                    Some(PengNativeCallable::Operation(operation_ntv.clone())),
                    false,
                    PengInstruction::OperationCall,
                    position,
                    None,
                )
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

            let mut ctx = PengNativeFunctionCallContext::new(env, thread, unit, args.clone());

            let ret = match ntv_call {
                PengNativeCallable::Function(ntv_fn) => match ntv_fn.call(&mut ctx) {
                    Ok(ret) => ret,
                    Err(e) => {
                        let e = e.push(PengError::CannotCallValue(
                            "native function call failed".to_string(),
                        ));

                        match env.recover_thread_try_error(thread) {
                            Ok(true) => return Ok(None),
                            Ok(false) => {
                                return Err(PengError::PositionedError {
                                    error: Box::new(e),
                                    position: instruction_position,
                                });
                            }
                            Err(recover_error) => {
                                return Err(PengError::PositionedError {
                                    error: Box::new(recover_error),
                                    position: instruction_position,
                                });
                            }
                        }
                    }
                },

                PengNativeCallable::Operation(ntv_oper) => {
                    let left = match args.get(0) {
                        Some(arg) => arg.clone(),
                        None => PengBinded::Mutable(PengCell::Nil),
                    };

                    let right = match args.get(1) {
                        Some(arg) => arg.clone(),
                        None => PengBinded::Mutable(PengCell::Nil),
                    };

                    let mut ctx =
                        PengNativeOperationCallContext::new(env, thread, unit, left, right);

                    match ntv_oper.call(&mut ctx) {
                        Ok(ret) => ret,
                        Err(e) => {
                            let e = e.push(PengError::CannotCallValue(
                                "native operation call failed".to_string(),
                            ));

                            match env.recover_thread_try_error(thread) {
                                Ok(true) => return Ok(None),
                                Ok(false) => {
                                    return Err(PengError::PositionedError {
                                        error: Box::new(e),
                                        position: instruction_position,
                                    });
                                }
                                Err(recover_error) => {
                                    return Err(PengError::PositionedError {
                                        error: Box::new(recover_error),
                                        position: instruction_position,
                                    });
                                }
                            }
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

            match execute_instruction(
                instruction.clone(),
                instruction_position.clone(),
                constant,
                thread,
                frame_base,
                env,
                unit,
            ) {
                Ok(res) => match res {
                    Some(ret) => {
                        let frame = match env.pop_thread_frame(thread) {
                            Ok(frame) => frame,
                            Err(e) => {
                                return Err(e.push(PengError::PositionedError {
                                    error: Box::new(PengError::InvalidInstruction(instruction)),
                                    position: instruction_position,
                                }));
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
                            return Err(PengError::PositionedError {
                                error: Box::new(e),
                                position: instruction_position,
                            });
                        }
                    }

                    Err(e) => {
                        return Err(PengError::PositionedError {
                            error: Box::new(e),
                            position: instruction_position,
                        });
                    }
                },
            };
        }
    }

    Ok(None)
}

pub fn call_function_sync(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    unit: &PengUnit,
    callable: PengBindedCell,
    args: Vec<PengBindedCell>,
) -> Result<PengBindedCell, PengError> {
    let initial_frames_len = match env.get_thread_frames_len(thread) {
        Ok(len) => len,
        Err(e) => return Err(e),
    };

    match env.push_thread_binded_stated_cell(thread, callable) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    for arg in &args {
        match env.push_thread_binded_stated_cell(thread, arg.clone()) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    match env.execute_function_call(thread, args.len(), false, None) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    loop {
        let frames_len = match env.get_thread_frames_len(thread) {
            Ok(len) => len,
            Err(e) => return Err(e),
        };

        if frames_len <= initial_frames_len {
            break;
        }

        match step_thread(env, thread, unit) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }

    let result = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
        Ok(result) => result,
        Err(e) => return Err(e),
    };

    match env.pop_thread_stack_n_times(thread, 1) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(result)
}

enum PengNativeCallable {
    Function(PengNativeFunction),
    Operation(PengNativeOperation),
}

#[derive(Clone, Copy)]
enum NumericBinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Remainder,
}

fn apply_float32_operation(
    left: f32,
    right: f32,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    match operation {
        NumericBinaryOperation::Add => Some(PengCell::Float32(left + right)),
        NumericBinaryOperation::Subtract => Some(PengCell::Float32(left - right)),
        NumericBinaryOperation::Multiply => Some(PengCell::Float32(left * right)),
        NumericBinaryOperation::Divide => {
            if right == 0.0 {
                None
            } else {
                Some(PengCell::Float32(left / right))
            }
        }
        NumericBinaryOperation::Power => Some(PengCell::Float32(left.powf(right))),
        NumericBinaryOperation::Remainder => {
            if right == 0.0 {
                None
            } else {
                Some(PengCell::Float32(left % right))
            }
        }
    }
}

fn apply_float64_operation(
    left: f64,
    right: f64,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    match operation {
        NumericBinaryOperation::Add => Some(PengCell::Float64(left + right)),
        NumericBinaryOperation::Subtract => Some(PengCell::Float64(left - right)),
        NumericBinaryOperation::Multiply => Some(PengCell::Float64(left * right)),
        NumericBinaryOperation::Divide => {
            if right == 0.0 {
                None
            } else {
                Some(PengCell::Float64(left / right))
            }
        }
        NumericBinaryOperation::Power => Some(PengCell::Float64(left.powf(right))),
        NumericBinaryOperation::Remainder => {
            if right == 0.0 {
                None
            } else {
                Some(PengCell::Float64(left % right))
            }
        }
    }
}

fn apply_int_operation(
    left: isize,
    right: isize,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    let value = match operation {
        NumericBinaryOperation::Add => left.checked_add(right),
        NumericBinaryOperation::Subtract => left.checked_sub(right),
        NumericBinaryOperation::Multiply => left.checked_mul(right),
        NumericBinaryOperation::Divide => left.checked_div(right),
        NumericBinaryOperation::Power => {
            if right < 0 {
                return Some(PengCell::Float32((left as f32).powf(right as f32)));
            }

            match u32::try_from(right) {
                Ok(exponent) => left.checked_pow(exponent),
                Err(_) => None,
            }
        }
        NumericBinaryOperation::Remainder => left.checked_rem(right),
    };

    match value {
        Some(value) => Some(PengCell::Int(value)),
        None => None,
    }
}

fn apply_uint_operation(
    left: usize,
    right: usize,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    let value = match operation {
        NumericBinaryOperation::Add => left.checked_add(right),
        NumericBinaryOperation::Subtract => left.checked_sub(right),
        NumericBinaryOperation::Multiply => left.checked_mul(right),
        NumericBinaryOperation::Divide => left.checked_div(right),
        NumericBinaryOperation::Power => match u32::try_from(right) {
            Ok(exponent) => left.checked_pow(exponent),
            Err(_) => None,
        },
        NumericBinaryOperation::Remainder => left.checked_rem(right),
    };

    match value {
        Some(value) => Some(PengCell::Uint(value)),
        None => None,
    }
}

fn apply_byte_operation(
    left: u8,
    right: u8,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    let value = match operation {
        NumericBinaryOperation::Add => left.checked_add(right),
        NumericBinaryOperation::Subtract => left.checked_sub(right),
        NumericBinaryOperation::Multiply => left.checked_mul(right),
        NumericBinaryOperation::Divide => left.checked_div(right),
        NumericBinaryOperation::Power => left.checked_pow(right as u32),
        NumericBinaryOperation::Remainder => left.checked_rem(right),
    };

    match value {
        Some(value) => Some(PengCell::Byte(value)),
        None => None,
    }
}

fn apply_mixed_int_operation(
    left: i128,
    right: i128,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    let value = match operation {
        NumericBinaryOperation::Add => left.checked_add(right),
        NumericBinaryOperation::Subtract => left.checked_sub(right),
        NumericBinaryOperation::Multiply => left.checked_mul(right),
        NumericBinaryOperation::Divide => left.checked_div(right),
        NumericBinaryOperation::Power => {
            if right < 0 {
                return Some(PengCell::Float32((left as f32).powf(right as f32)));
            }

            match u32::try_from(right) {
                Ok(exponent) => left.checked_pow(exponent),
                Err(_) => None,
            }
        }
        NumericBinaryOperation::Remainder => left.checked_rem(right),
    };

    match value {
        Some(value) => match isize::try_from(value) {
            Ok(value) => Some(PengCell::Int(value)),
            Err(_) => None,
        },
        None => None,
    }
}

fn binary_numeric_cells(
    left: &PengCell,
    right: &PengCell,
    operation: NumericBinaryOperation,
) -> Option<PengCell> {
    match (left, right) {
        (PengCell::Float64(left), PengCell::Float64(right)) => {
            apply_float64_operation(*left, *right, operation)
        }
        (PengCell::Float64(left), PengCell::Float32(right)) => {
            apply_float64_operation(*left, *right as f64, operation)
        }
        (PengCell::Float64(left), PengCell::Int(right)) => {
            apply_float64_operation(*left, *right as f64, operation)
        }
        (PengCell::Float64(left), PengCell::Uint(right)) => {
            apply_float64_operation(*left, *right as f64, operation)
        }
        (PengCell::Float64(left), PengCell::Byte(right)) => {
            apply_float64_operation(*left, *right as f64, operation)
        }
        (PengCell::Float32(left), PengCell::Float64(right)) => {
            apply_float64_operation(*left as f64, *right, operation)
        }
        (PengCell::Int(left), PengCell::Float64(right)) => {
            apply_float64_operation(*left as f64, *right, operation)
        }
        (PengCell::Uint(left), PengCell::Float64(right)) => {
            apply_float64_operation(*left as f64, *right, operation)
        }
        (PengCell::Byte(left), PengCell::Float64(right)) => {
            apply_float64_operation(*left as f64, *right, operation)
        }

        (PengCell::Float32(left), PengCell::Float32(right)) => {
            apply_float32_operation(*left, *right, operation)
        }
        (PengCell::Float32(left), PengCell::Int(right)) => {
            apply_float32_operation(*left, *right as f32, operation)
        }
        (PengCell::Float32(left), PengCell::Uint(right)) => {
            apply_float32_operation(*left, *right as f32, operation)
        }
        (PengCell::Float32(left), PengCell::Byte(right)) => {
            apply_float32_operation(*left, *right as f32, operation)
        }
        (PengCell::Int(left), PengCell::Float32(right)) => {
            apply_float32_operation(*left as f32, *right, operation)
        }
        (PengCell::Uint(left), PengCell::Float32(right)) => {
            apply_float32_operation(*left as f32, *right, operation)
        }
        (PengCell::Byte(left), PengCell::Float32(right)) => {
            apply_float32_operation(*left as f32, *right, operation)
        }

        (PengCell::Int(left), PengCell::Int(right)) => {
            apply_int_operation(*left, *right, operation)
        }
        (PengCell::Int(left), PengCell::Uint(right)) => match i128::try_from(*right) {
            Ok(right) => apply_mixed_int_operation(*left as i128, right, operation),
            Err(_) => None,
        },
        (PengCell::Uint(left), PengCell::Int(right)) => match i128::try_from(*left) {
            Ok(left) => apply_mixed_int_operation(left, *right as i128, operation),
            Err(_) => None,
        },
        (PengCell::Int(left), PengCell::Byte(right)) => {
            apply_int_operation(*left, *right as isize, operation)
        }
        (PengCell::Byte(left), PengCell::Int(right)) => {
            apply_int_operation(*left as isize, *right, operation)
        }
        (PengCell::Uint(left), PengCell::Uint(right)) => {
            apply_uint_operation(*left, *right, operation)
        }
        (PengCell::Uint(left), PengCell::Byte(right)) => {
            apply_uint_operation(*left, *right as usize, operation)
        }
        (PengCell::Byte(left), PengCell::Uint(right)) => {
            apply_uint_operation(*left as usize, *right, operation)
        }
        (PengCell::Byte(left), PengCell::Byte(right)) => {
            apply_byte_operation(*left, *right, operation)
        }

        _ => None,
    }
}

fn resolve_numeric_cell(env: &PengEnv, cell: &PengCell) -> Result<Option<PengCell>, PengError> {
    match cell {
        PengCell::Int(_)
        | PengCell::Uint(_)
        | PengCell::Byte(_)
        | PengCell::Float32(_)
        | PengCell::Float64(_) => Ok(Some(cell.clone())),

        PengCell::Reference(ptr) => match env.get_heap(*ptr) {
            Some(PengValue::Cell(value)) => match value {
                PengCell::Int(_)
                | PengCell::Uint(_)
                | PengCell::Byte(_)
                | PengCell::Float32(_)
                | PengCell::Float64(_) => Ok(Some(value.clone())),

                _ => Ok(None),
            },

            Some(_) => Ok(None),

            None => Err(PengError::HeapValueNotFound(*ptr)),
        },

        _ => Ok(None),
    }
}

fn execute_binary_numeric_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
    operation: NumericBinaryOperation,
) -> Result<bool, PengError> {
    let (left, right) = match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
        Ok(cells) => cells,
        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let left_cell = match resolve_numeric_cell(env, left.value()) {
        Ok(Some(cell)) => cell,

        Ok(None) => {
            return Ok(false);
        }

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let right_cell = match resolve_numeric_cell(env, right.value()) {
        Ok(Some(cell)) => cell,

        Ok(None) => {
            return Ok(false);
        }

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let result = match binary_numeric_cells(&left_cell, &right_cell, operation) {
        Some(result) => result,

        None => {
            return Err(PengError::InvalidInstruction(instruction));
        }
    };

    match env.pop_thread_stack_n_times(thread, 2) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(result)) {
        Ok(()) => Ok(true),

        Err(e) => Err(e.push(PengError::InvalidInstruction(instruction))),
    }
}

fn execute_custom_binary_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    unit: &PengUnit,
    instruction: PengInstruction,
    instruction_position: PengPosition,
    function: PengNativeFunction,
) -> Result<(), PengError> {
    let (left, right) = match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
        Ok(cells) => cells,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let args = vec![left.clone(), right.clone()];

    let mut ctx = PengNativeFunctionCallContext::new(env, thread, unit, args);

    let callable = match function.call(&mut ctx) {
        Ok(callable) => callable,

        Err(e) => {
            return Err(e
                .push(PengError::CannotCallValue(
                    "custom binary operation resolution failed".to_string(),
                ))
                .push(PengError::InvalidInstruction(instruction)));
        }
    };

    let callable_ptr = match callable.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(
                "custom binary operation expected callable".to_string(),
            )
            .push(PengError::InvalidInstruction(instruction)));
        }
    };

    match env.get_heap(callable_ptr) {
        Some(PengValue::Box(PengBox::Function(_))) => {}

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "custom binary operation expected function".to_string(),
            )
            .push(PengError::InvalidInstruction(instruction)));
        }

        None => {
            return Err(PengError::HeapValueNotFound(callable_ptr));
        }
    }

    match env.pop_thread_stack_n_times(thread, 2) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, callable) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, left) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, right) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.execute_function_call(thread, 2, false, Some(instruction_position)) {
        Ok(()) => Ok(()),

        Err(e) => Err(e.push(PengError::InvalidInstruction(instruction))),
    }
}

fn execute_custom_unary_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    unit: &PengUnit,
    instruction: PengInstruction,
    instruction_position: PengPosition,
    function: PengNativeFunction,
) -> Result<(), PengError> {
    let value = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
        Ok(cell) => cell,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let args = vec![value.clone()];

    let mut ctx = PengNativeFunctionCallContext::new(env, thread, unit, args);

    let callable = match function.call(&mut ctx) {
        Ok(callable) => callable,

        Err(e) => {
            return Err(e
                .push(PengError::CannotCallValue(
                    "custom unary operation resolution failed".to_string(),
                ))
                .push(PengError::InvalidInstruction(instruction)));
        }
    };

    let callable_ptr = match callable.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(
                "custom unary operation expected callable".to_string(),
            )
            .push(PengError::InvalidInstruction(instruction)));
        }
    };

    match env.get_heap(callable_ptr) {
        Some(PengValue::Box(PengBox::Function(_))) => {}

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "custom unary operation expected function".to_string(),
            )
            .push(PengError::InvalidInstruction(instruction)));
        }

        None => {
            return Err(PengError::HeapValueNotFound(callable_ptr));
        }
    }

    match env.pop_thread_stack_n_times(thread, 1) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, callable) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, value) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.execute_function_call(thread, 1, false, Some(instruction_position)) {
        Ok(()) => Ok(()),

        Err(e) => Err(e.push(PengError::InvalidInstruction(instruction))),
    }
}

fn push_thread_operation_result(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
    pops: usize,
    cell: PengCell,
) -> Result<(), PengError> {
    match env.pop_thread_stack_n_times(thread, pops) {
        Ok(()) => {}

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    }

    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell)) {
        Ok(()) => Ok(()),

        Err(e) => Err(e.push(PengError::InvalidInstruction(instruction))),
    }
}

fn get_binary_operation_values(
    env: &PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
) -> Result<(PengValue, PengValue), PengError> {
    let (left_cell, right_cell) = match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
        Ok(cells) => cells,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let left = match env.get_value_from_cell(left_cell.value().clone()) {
        Ok(value) => value,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let right = match env.get_value_from_cell(right_cell.value().clone()) {
        Ok(value) => value,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    Ok((left, right))
}

fn execute_unary_negate_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
) -> Result<bool, PengError> {
    let cell = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
        Ok(cell) => cell,
        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let value = match cell.value() {
        PengCell::Int(v) => match v.checked_neg() {
            Some(value) => PengCell::Int(value),
            None => {
                return Err(PengError::InvalidInstruction(instruction));
            }
        },
        PengCell::Float32(v) => PengCell::Float32(-v),
        PengCell::Float64(v) => PengCell::Float64(-v),
        _ => {
            return Ok(false);
        }
    };

    match push_thread_operation_result(env, thread, instruction, 1, value) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

fn execute_unary_not_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
) -> Result<bool, PengError> {
    let cell = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
        Ok(cell) => cell,
        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let value = match cell.value() {
        PengCell::Bool(value) => *value,
        _ => {
            return Ok(false);
        }
    };

    match push_thread_operation_result(env, thread, instruction, 1, PengCell::Bool(!value)) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

fn execute_binary_concat_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
) -> Result<bool, PengError> {
    let (left, right) = match get_binary_operation_values(env, thread, instruction.clone()) {
        Ok(values) => values,
        Err(e) => {
            return Err(e);
        }
    };

    let value = match (left, right) {
        (PengValue::Box(PengBox::String(a)), PengValue::Box(PengBox::String(b))) => {
            PengValue::Box(PengBox::String(format!("{}{}", a, b)))
        }

        _ => {
            return Ok(false);
        }
    };

    let cell = match env.get_cell_from_value(value) {
        Ok(cell) => cell,
        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    match push_thread_operation_result(env, thread, instruction, 2, cell) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

fn execute_binary_bool_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
    operation: fn(bool, bool) -> bool,
) -> Result<bool, PengError> {
    let (left_cell, right_cell) = match env.get_thread_2_latests_binded_stated_cell_cloned(thread) {
        Ok(cells) => cells,

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    let left = match left_cell.value() {
        PengCell::Bool(value) => *value,
        _ => {
            return Ok(false);
        }
    };

    let right = match right_cell.value() {
        PengCell::Bool(value) => *value,
        _ => {
            return Ok(false);
        }
    };

    match push_thread_operation_result(
        env,
        thread,
        instruction,
        2,
        PengCell::Bool(operation(left, right)),
    ) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

fn value_supports_builtin_equals(value: &PengValue) -> bool {
    match value {
        PengValue::Cell(cell) => match cell {
            PengCell::Nil
            | PengCell::Int(_)
            | PengCell::Uint(_)
            | PengCell::Float32(_)
            | PengCell::Float64(_)
            | PengCell::Byte(_)
            | PengCell::Bool(_) => true,

            PengCell::Reference(_) => false,
        },

        PengValue::Box(value) => match value {
            PengBox::String(_) | PengBox::Type(_) => true,

            _ => false,
        },
    }
}

fn execute_binary_equals_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
    invert: bool,
) -> Result<bool, PengError> {
    let (left, right) = match get_binary_operation_values(env, thread, instruction.clone()) {
        Ok(values) => values,
        Err(e) => {
            return Err(e);
        }
    };

    if !value_supports_builtin_equals(&left) || !value_supports_builtin_equals(&right) {
        return Ok(false);
    }

    let result = if invert {
        !left.equals(&right)
    } else {
        left.equals(&right)
    };

    match push_thread_operation_result(env, thread, instruction, 2, PengCell::Bool(result)) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

fn execute_binary_comparison_operation(
    env: &mut PengEnv,
    thread: PengHeapPtr,
    instruction: PengInstruction,
    operation: fn(&PengValue, &PengValue) -> Result<bool, PengError>,
) -> Result<bool, PengError> {
    let (left, right) = match get_binary_operation_values(env, thread, instruction.clone()) {
        Ok(values) => values,
        Err(e) => {
            return Err(e);
        }
    };

    let result = match operation(&left, &right) {
        Ok(result) => result,

        Err(PengError::InvalidBinaryOperationValue { .. })
        | Err(PengError::InvalidBinaryOperationCell { .. }) => {
            return Ok(false);
        }

        Err(e) => {
            return Err(e.push(PengError::InvalidInstruction(instruction)));
        }
    };

    match push_thread_operation_result(env, thread, instruction, 2, PengCell::Bool(result)) {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

pub fn execute_instruction(
    instruction: PengInstruction,
    instruction_position: PengPosition,
    constant: Option<PengValue>,
    thread: PengHeapPtr,
    frame_base: usize,
    env: &mut PengEnv,
    unit: &PengUnit,
) -> Result<Option<PengBindedCell>, PengError> {
    match instruction {
        PengInstruction::MakeImmutable => {
            let cell = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
                    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell)) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                PengValue::Box(value) => {
                    let ptr = env.create_heap_value(PengValue::Box(value));
                    let cell = PengCell::Reference(ptr);

                    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell)) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }
            }
        }

        PengInstruction::PushLocal(local) => match env.get_thread_local(thread, local).cloned() {
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

        PengInstruction::ReserveLocal(local) => match env.reserve_thread_local(thread, local) {
            Ok(()) => {}
            Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
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

            let value = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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

            match value {
                PengValue::Cell(cell) => {
                    match env.push_thread_binded_stated_cell(thread, PengBinded::Mutable(cell)) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidInstruction(instruction)));
                        }
                    }
                }

                PengValue::Box(_) => {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(value_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
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
                                        return Err(
                                            e.push(PengError::InvalidInstruction(instruction))
                                        );
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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

                        let bindedcell = value_cell.clone();

                        match env.get_heap_mut(object_ptr) {
                            Some(PengValue::Box(PengBox::Object(object))) => {
                                if let Some(existing) = object.fields.get(&name_ptr) {
                                    if matches!(existing, PengBinded::Immutable(_)) {
                                        return Err(PengError::CannotMutateImmutable);
                                    }
                                }

                                object.fields.insert(name_ptr, bindedcell);
                            }

                            Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
                                if let Some(existing) = custom_type.fields.get(&name_ptr) {
                                    if matches!(existing, PengBinded::Immutable(_)) {
                                        return Err(PengError::CannotMutateImmutable);
                                    }
                                }

                                custom_type.fields.insert(name_ptr, bindedcell);
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(object_cell) => {
                    let object_ptr = match object_cell.value() {
                        PengCell::Reference(ptr) => *ptr,
                        _ => return Err(PengError::ExpectedReference),
                    };

                    let value = match env.get_heap(object_ptr) {
                        Some(PengValue::Box(PengBox::Object(object))) => {
                            match object.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => match custom_access_as_cell(env, unit, name_ptr) {
                                    Some(value) => value,
                                    None => return Err(PengError::AttributeNotFound(name_ptr)),
                                },
                            }
                        }

                        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
                            match custom_type.fields.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => match custom_access_as_cell(env, unit, name_ptr) {
                                    Some(value) => value,
                                    None => return Err(PengError::AttributeNotFound(name_ptr)),
                                },
                            }
                        }

                        Some(PengValue::Box(PengBox::Module(module))) => {
                            match module.members.get(&name_ptr) {
                                Some(value) => value.clone(),
                                None => match custom_access_as_cell(env, unit, name_ptr) {
                                    Some(value) => value,
                                    None => return Err(PengError::AttributeNotFound(name_ptr)),
                                },
                            }
                        }

                        Some(_) => match custom_access_as_cell(env, unit, name_ptr) {
                            Some(value) => value,
                            None => return Err(PengError::AttributeNotFound(name_ptr)),
                        },

                        None => return Err(PengError::HeapValueNotFound(object_ptr)),
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(value) => {
                    return Ok(Some(value));
                }

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }
        PengInstruction::FunctionCall(args_count) => {
            match env.execute_function_call(
                thread,
                args_count,
                false,
                Some(instruction_position.clone()),
            ) {
                Ok(()) => return Ok(None),
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

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

        PengInstruction::Convert => {
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(target_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
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

        PengInstruction::Add => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Add,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_add() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {
                        return Ok(None);
                    }

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },
        PengInstruction::Subtract => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Subtract,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_subtract() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {
                        return Ok(None);
                    }

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },
        PengInstruction::Multiply => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Multiply,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_multiply() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::Divide => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Divide,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_divide() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::Power => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Power,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_power() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::Remainder => match execute_binary_numeric_operation(
            env,
            thread,
            instruction.clone(),
            NumericBinaryOperation::Remainder,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_remainder() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::Negate => {
            match execute_unary_negate_operation(env, thread, instruction.clone()) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_negate() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_unary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::Concat => {
            match execute_binary_concat_operation(env, thread, instruction.clone()) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_concat() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_binary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::And => {
            match execute_binary_bool_operation(env, thread, instruction.clone(), |left, right| {
                left && right
            }) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_and() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_binary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::Or => {
            match execute_binary_bool_operation(env, thread, instruction.clone(), |left, right| {
                left || right
            }) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_or() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_binary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::Not => match execute_unary_not_operation(env, thread, instruction.clone())
        {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_not() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_unary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::Equals => {
            match execute_binary_equals_operation(env, thread, instruction.clone(), false) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_equals() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_binary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::NotEquals => {
            match execute_binary_equals_operation(env, thread, instruction.clone(), true) {
                Ok(true) => {}

                Ok(false) => {
                    let custom = match unit.custom_not_equals() {
                        Some(custom) => custom.clone(),

                        None => {
                            return Err(PengError::InvalidInstruction(instruction));
                        }
                    };

                    match execute_custom_binary_operation(
                        env,
                        thread,
                        unit,
                        instruction,
                        instruction_position.clone(),
                        custom,
                    ) {
                        Ok(()) => {}

                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }

        PengInstruction::TryFunctionCall(args_count) => {
            match env.execute_function_call(
                thread,
                args_count,
                true,
                Some(instruction_position.clone()),
            ) {
                Ok(()) => return Ok(None),
                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }
        }

        PengInstruction::GreaterThan => match execute_binary_comparison_operation(
            env,
            thread,
            instruction.clone(),
            PengValue::greater_than,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_greater_than() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::GreaterEqualsThan => match execute_binary_comparison_operation(
            env,
            thread,
            instruction.clone(),
            PengValue::greater_equals_than,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_greater_equals_than() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::LessThan => match execute_binary_comparison_operation(
            env,
            thread,
            instruction.clone(),
            PengValue::less_than,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_less_than() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::LessEqualsThan => match execute_binary_comparison_operation(
            env,
            thread,
            instruction.clone(),
            PengValue::less_equals_than,
        ) {
            Ok(true) => {}

            Ok(false) => {
                let custom = match unit.custom_less_equals_than() {
                    Some(custom) => custom.clone(),

                    None => {
                        return Err(PengError::InvalidInstruction(instruction));
                    }
                };

                match execute_custom_binary_operation(
                    env,
                    thread,
                    unit,
                    instruction,
                    instruction_position.clone(),
                    custom,
                ) {
                    Ok(()) => {}

                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(e) => {
                return Err(e);
            }
        },

        PengInstruction::OperationCall => {
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(right_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
                        Ok(left_cell) => {
                            match env.get_thread_latest_binded_stated_cell(thread, 2).cloned() {
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(index_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
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

                            let vector_index = match &index_value {
                                PengValue::Cell(PengCell::Int(v)) => {
                                    if *v < 0 {
                                        return Err(PengError::InvalidIndexTypeValue(
                                            index_value.clone(),
                                        ));
                                    }

                                    Some(*v as usize)
                                }

                                PengValue::Cell(PengCell::Uint(v)) => Some(*v),

                                _ => None,
                            };

                            let object_key = match &index_value {
                                PengValue::Box(PengBox::String(key)) => {
                                    Some(env.ensure_pooled_name_ptr(key.clone()))
                                }

                                _ => None,
                            };

                            let mut object_value = match object_cell.value() {
                                PengCell::Reference(ptr) => match env.get_heap(*ptr) {
                                    Some(value) => value.clone(),
                                    None => {
                                        return Err(PengError::HeapValueNotFound(*ptr));
                                    }
                                },

                                value => PengValue::Cell(value.clone()),
                            };

                            loop {
                                let next_ptr = match &object_value {
                                    PengValue::Cell(PengCell::Reference(ptr)) => Some(*ptr),
                                    _ => None,
                                };

                                let next_ptr = match next_ptr {
                                    Some(ptr) => ptr,
                                    None => break,
                                };

                                object_value = match env.get_heap(next_ptr) {
                                    Some(value) => value.clone(),
                                    None => {
                                        return Err(PengError::HeapValueNotFound(next_ptr));
                                    }
                                };
                            }

                            let value = match object_value {
                                PengValue::Box(PengBox::Vector(vector)) => {
                                    if let Some(name_ptr) = object_key {
                                        match custom_access_as_cell(env, unit, name_ptr) {
                                            Some(value) => value,
                                            None => {
                                                return Err(PengError::AttributeNotFound(name_ptr));
                                            }
                                        }
                                    } else {
                                        let index = match vector_index {
                                            Some(index) => index,
                                            None => {
                                                return Err(PengError::InvalidIndexTypeValue(
                                                    index_value,
                                                ));
                                            }
                                        };

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
                                }

                                PengValue::Box(PengBox::String(text)) => {
                                    if let Some(name_ptr) = object_key {
                                        match custom_access_as_cell(env, unit, name_ptr) {
                                            Some(value) => value,
                                            None => {
                                                return Err(PengError::AttributeNotFound(name_ptr));
                                            }
                                        }
                                    } else {
                                        let index = match vector_index {
                                            Some(index) => index,
                                            None => {
                                                return Err(PengError::InvalidIndexTypeValue(
                                                    index_value,
                                                ));
                                            }
                                        };

                                        let ch = match text.chars().nth(index) {
                                            Some(ch) => ch,
                                            None => {
                                                return Err(PengError::IndexOutOfBounds {
                                                    index,
                                                    len: text.chars().count(),
                                                });
                                            }
                                        };

                                        let ptr = env.create_heap_value(PengValue::Box(
                                            PengBox::String(ch.to_string()),
                                        ));

                                        PengBinded::Mutable(PengCell::Reference(ptr))
                                    }
                                }

                                PengValue::Box(PengBox::Object(object)) => {
                                    let name_ptr = match object_key {
                                        Some(name_ptr) => name_ptr,
                                        None => {
                                            return Err(PengError::InvalidIndexTypeValue(
                                                index_value,
                                            ));
                                        }
                                    };

                                    match object.fields.get(&name_ptr) {
                                        Some(value) => value.clone(),
                                        None => match custom_access_as_cell(env, unit, name_ptr) {
                                            Some(value) => value,
                                            None => {
                                                return Err(PengError::AttributeNotFound(name_ptr));
                                            }
                                        },
                                    }
                                }

                                PengValue::Box(PengBox::Type(PengType::Custom(custom_type))) => {
                                    let name_ptr = match object_key {
                                        Some(name_ptr) => name_ptr,
                                        None => {
                                            return Err(PengError::InvalidIndexTypeValue(
                                                index_value,
                                            ));
                                        }
                                    };

                                    match custom_type.fields.get(&name_ptr) {
                                        Some(value) => value.clone(),
                                        None => match custom_access_as_cell(env, unit, name_ptr) {
                                            Some(value) => value,
                                            None => {
                                                return Err(PengError::AttributeNotFound(name_ptr));
                                            }
                                        },
                                    }
                                }

                                PengValue::Box(PengBox::Module(module)) => {
                                    let name_ptr = match object_key {
                                        Some(name_ptr) => name_ptr,
                                        None => {
                                            return Err(PengError::InvalidIndexTypeValue(
                                                index_value,
                                            ));
                                        }
                                    };

                                    match module.members.get(&name_ptr) {
                                        Some(value) => value.clone(),
                                        None => match custom_access_as_cell(env, unit, name_ptr) {
                                            Some(value) => value,
                                            None => {
                                                return Err(PengError::AttributeNotFound(name_ptr));
                                            }
                                        },
                                    }
                                }

                                _ => {
                                    let name_ptr = match object_key {
                                        Some(name_ptr) => name_ptr,
                                        None => {
                                            return Err(PengError::CannotIndexValue(
                                                "non-indexable value".to_string(),
                                            ));
                                        }
                                    };

                                    match custom_access_as_cell(env, unit, name_ptr) {
                                        Some(value) => value,
                                        None => {
                                            return Err(PengError::AttributeNotFound(name_ptr));
                                        }
                                    }
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
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(value_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
                        Ok(index_cell) => {
                            match env.get_thread_latest_binded_stated_cell(thread, 2).cloned() {
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

                                    let mut object_ptr = match object_cell.value() {
                                        PengCell::Reference(ptr) => *ptr,
                                        _ => {
                                            return Err(PengError::ExpectedReference);
                                        }
                                    };

                                    loop {
                                        let next_ptr = match env.get_heap(object_ptr) {
                                            Some(PengValue::Cell(PengCell::Reference(ptr))) => {
                                                Some(*ptr)
                                            }
                                            Some(_) => None,
                                            None => {
                                                return Err(PengError::HeapValueNotFound(
                                                    object_ptr,
                                                ));
                                            }
                                        };

                                        match next_ptr {
                                            Some(ptr) => object_ptr = ptr,
                                            None => break,
                                        }
                                    }

                                    let string_replacement = match env.get_heap(object_ptr) {
                                        Some(PengValue::Box(PengBox::String(_))) => {
                                            match env
                                                .get_value_from_cell(value_cell.value().clone())
                                            {
                                                Ok(PengValue::Box(PengBox::String(value))) => {
                                                    if value.chars().count() != 1 {
                                                        return Err(PengError::InvalidInstruction(
                                                            instruction,
                                                        ));
                                                    }

                                                    match value.chars().next() {
                                                        Some(ch) => Some(ch),
                                                        None => {
                                                            return Err(
                                                                PengError::InvalidInstruction(
                                                                    instruction,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                }

                                                Ok(_) => {
                                                    return Err(PengError::InvalidInstruction(
                                                        instruction,
                                                    ));
                                                }

                                                Err(e) => {
                                                    return Err(e.push(
                                                        PengError::InvalidInstruction(instruction),
                                                    ));
                                                }
                                            }
                                        }

                                        Some(_) => None,
                                        None => {
                                            return Err(PengError::HeapValueNotFound(object_ptr));
                                        }
                                    };

                                    let vector_index = match &index_value {
                                        PengValue::Cell(PengCell::Int(v)) => {
                                            if *v < 0 {
                                                return Err(PengError::InvalidIndexTypeValue(
                                                    index_value.clone(),
                                                ));
                                            }

                                            Some(*v as usize)
                                        }

                                        PengValue::Cell(PengCell::Uint(v)) => Some(*v),

                                        _ => None,
                                    };

                                    let object_key = match &index_value {
                                        PengValue::Box(PengBox::String(key)) => {
                                            Some(env.ensure_pooled_name_ptr(key.clone()))
                                        }

                                        _ => None,
                                    };

                                    match env.get_heap_mut(object_ptr) {
                                        Some(PengValue::Box(PengBox::Vector(vector))) => {
                                            let index = match vector_index {
                                                Some(index) => index,
                                                None => {
                                                    return Err(PengError::InvalidIndexTypeValue(
                                                        index_value,
                                                    ));
                                                }
                                            };

                                            if index >= vector.values.len() {
                                                return Err(PengError::IndexOutOfBounds {
                                                    index,
                                                    len: vector.values.len(),
                                                });
                                            }

                                            vector.values[index] = value_cell;
                                        }

                                        Some(PengValue::Box(PengBox::String(value))) => {
                                            let index = match vector_index {
                                                Some(index) => index,
                                                None => {
                                                    return Err(PengError::InvalidIndexTypeValue(
                                                        index_value,
                                                    ));
                                                }
                                            };

                                            let mut chars: Vec<char> = value.chars().collect();

                                            if index >= chars.len() {
                                                return Err(PengError::IndexOutOfBounds {
                                                    index,
                                                    len: chars.len(),
                                                });
                                            }

                                            let replacement_char = match string_replacement {
                                                Some(ch) => ch,
                                                None => {
                                                    return Err(PengError::InvalidInstruction(
                                                        instruction,
                                                    ));
                                                }
                                            };

                                            chars[index] = replacement_char;
                                            *value = chars.into_iter().collect();
                                        }

                                        Some(PengValue::Box(PengBox::Object(object))) => {
                                            let name_ptr = match object_key {
                                                Some(name_ptr) => name_ptr,
                                                None => {
                                                    return Err(PengError::InvalidIndexTypeValue(
                                                        index_value,
                                                    ));
                                                }
                                            };

                                            if let Some(existing) = object.fields.get(&name_ptr) {
                                                if matches!(existing, PengBinded::Immutable(_)) {
                                                    return Err(PengError::CannotMutateImmutable);
                                                }
                                            }

                                            object.fields.insert(name_ptr, value_cell);
                                        }

                                        Some(PengValue::Box(PengBox::Type(PengType::Custom(
                                            custom_type,
                                        )))) => {
                                            let name_ptr = match object_key {
                                                Some(name_ptr) => name_ptr,
                                                None => {
                                                    return Err(PengError::InvalidIndexTypeValue(
                                                        index_value,
                                                    ));
                                                }
                                            };

                                            if let Some(existing) =
                                                custom_type.fields.get(&name_ptr)
                                            {
                                                if matches!(existing, PengBinded::Immutable(_)) {
                                                    return Err(PengError::CannotMutateImmutable);
                                                }
                                            }

                                            custom_type.fields.insert(name_ptr, value_cell);
                                        }

                                        Some(PengValue::Box(PengBox::Module(module))) => {
                                            let name_ptr = match object_key {
                                                Some(name_ptr) => name_ptr,
                                                None => {
                                                    return Err(PengError::InvalidIndexTypeValue(
                                                        index_value,
                                                    ));
                                                }
                                            };

                                            if let Some(existing) = module.members.get(&name_ptr) {
                                                if matches!(existing, PengBinded::Immutable(_)) {
                                                    return Err(PengError::CannotMutateImmutable);
                                                }
                                            }

                                            module.members.insert(name_ptr, value_cell);
                                        }

                                        Some(PengValue::Box(PengBox::Type(_))) => {
                                            return Err(PengError::ExpectedType);
                                        }

                                        Some(_) => {
                                            return Err(PengError::CannotSetIndex(
                                                "non-indexable heap value".to_string(),
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
                        return Err(PengError::InvalidInstruction(instruction));
                    }

                    let spread_cell =
                        match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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
                        instruction_position.clone(),
                        None,
                        thread,
                        frame_base,
                        env,
                        unit,
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
                        return Err(PengError::InvalidInstruction(instruction));
                    }

                    let spread_cell =
                        match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
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

                    match env.execute_function_call(
                        thread,
                        args_count,
                        true,
                        Some(instruction_position.clone()),
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

        PengInstruction::TryOperationCall => {
            match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(right_cell) => {
                    match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
                        Ok(left_cell) => {
                            match env.get_thread_latest_binded_stated_cell(thread, 2).cloned() {
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
        PengInstruction::Swap => {
            let a = match env.get_thread_latest_binded_stated_cell(thread, 0).cloned() {
                Ok(v) => v,
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            };

            let b = match env.get_thread_latest_binded_stated_cell(thread, 1).cloned() {
                Ok(v) => v,
                Err(e) => return Err(e.push(PengError::InvalidInstruction(instruction))),
            };

            match env.pop_thread_stack_n_times(thread, 2) {
                Ok(()) => {}

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }

            match env.push_thread_binded_stated_cell(thread, a) {
                Ok(()) => {}

                Err(e) => {
                    return Err(e.push(PengError::InvalidInstruction(instruction)));
                }
            }

            match env.push_thread_binded_stated_cell(thread, b) {
                Ok(()) => {}

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

fn custom_access_as_cell(
    env: &mut PengEnv,
    unit: &PengUnit,
    name: PengNamePoolPtr,
) -> Option<PengBindedCell> {
    let ntv = match unit.custom_access().get(&name).cloned() {
        Some(v) => v,
        None => return None,
    };

    let ptr = env.create_heap_value(PengValue::Box(PengBox::Function(PengFunction::Native(ntv))));

    env.pinned_mut().insert(ptr);

    Some(PengBinded::Immutable(PengCell::Reference(ptr)))
}
