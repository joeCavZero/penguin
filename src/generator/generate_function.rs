use crate::core::*;
pub use crate::generator::generate::*;

use crate::parser::*;

pub fn create_anonymous_bytecode_function(
    env: &mut PengEnv,
    context: PengGeneratorContext,
    params: PengBytecodeFunctionParams,
    pos: PengPosition,
) -> PengHeapPtr {
    let function = create_bytecode_function_value(env, context, params, pos);

    env.create_heap_value(PengValue::Box(function))
}

pub fn create_bytecode_function_value(
    env: &mut PengEnv,
    mut context: PengGeneratorContext,
    params: PengBytecodeFunctionParams,
    pos: PengPosition,
) -> PengBox {
    context.prepend_frame_local_reserves(pos.clone());
    context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil), pos.clone());
    context.push_positioned_instruction(PengInstruction::Return, pos);
    debug_assert_eq!(context.bytecode.len(), context.positions.len());

    PengBox::Function(PengFunction::Bytecode(PengBytecodeFunction {
        bytecode: context.bytecode,
        positions: context.positions,
        consts: context.consts,
        using_values: Vec::new(),
        params,
    }))
}

pub fn generate_function_declaration_value(
    env: &mut PengEnv,
    context: &PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<PengBox, PengError> {
    generate_function_value(
        env,
        context,
        &declaration.value.params,
        &declaration.value.body,
        declaration.position.clone(),
    )
}

pub fn generate_function_value(
    env: &mut PengEnv,
    parent_context: &PengGeneratorContext,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
    pos: PengPosition,
) -> Result<PengBox, PengError> {
    let mut context = parent_context.new_child_context();

    let mut variadic_index: Option<usize> = None;

    for i in 0..params.len() {
        if params[i].value.variadic {
            if variadic_index.is_some() {
                return Err(PengError::InvalidState(
                    "function cannot have more than one variadic parameter".to_string(),
                ));
            }

            variadic_index = Some(i);
        }
    }

    let function_params = match variadic_index {
        Some(index) => {
            if index + 1 != params.len() {
                return Err(PengError::InvalidState(
                    "variadic parameter must be the last parameter".to_string(),
                ));
            }

            PengBytecodeFunctionParams::Variadic(index)
        }

        None => PengBytecodeFunctionParams::Fixed(params.len()),
    };

    for param in params {
        context.create_local(param.value.name.value.clone());
    }
    context.mark_frame_prefix_locals();

    match generate_statements(env, &mut context, body) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_function".to_string(),
            )));
        }
    }

    Ok(create_bytecode_function_value(
        env,
        context,
        function_params,
        pos,
    ))
}

pub fn generate_function_call(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    call: &PengFuncCallExpression,
    pos: PengPosition,
) -> Result<(), PengError> {
    match generate_expression(env, context, &call.function) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_function_call function".to_string(),
            )));
        }
    }

    let variadic_index = match generate_function_call_args(env, context, &call.args) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_function_call args".to_string(),
            )));
        }
    };

    match variadic_index {
        Some(index) => {
            context.push_positioned_instruction(PengInstruction::FunctionCallSpread(index), pos);
        }

        None => {
            context
                .push_positioned_instruction(PengInstruction::FunctionCall(call.args.len()), pos);
        }
    }

    Ok(())
}

pub fn generate_local_function_declaration(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    let global = get_allocated_global(env, context, &declaration.value.name.value);

    match global {
        Some(value_ptr) => {
            let value = match generate_function_declaration_value(env, context, declaration) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_function".to_string(),
                    )));
                }
            };

            context.push_positioned_instruction(
                PengInstruction::PushHeapRef(*value_ptr.value()),
                declaration.position.clone(),
            );

            context.push_const_and_const_instruction(
                env,
                PengValue::Box(value),
                declaration.position.clone(),
            );

            context.push_positioned_instruction(
                PengInstruction::StoreHeap,
                declaration.position.clone(),
            );

            Ok(())
        }

        None => {
            let value = match generate_function_declaration_value(env, context, declaration) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_function".to_string(),
                    )));
                }
            };

            context.push_const_and_const_instruction(
                env,
                PengValue::Box(value),
                declaration.position.clone(),
            );

            generate_make_immutable_if_needed(context, immutable, declaration.position.clone());

            let local = context.create_local(declaration.value.name.value.clone());

            context.push_positioned_instruction(
                PengInstruction::StoreLocal(local),
                declaration.position.clone(),
            );

            Ok(())
        }
    }
}

pub fn generate_function_call_args(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    args: &Vec<PengPositionedFunctionCallArg>,
) -> Result<Option<usize>, PengError> {
    let mut variadic_index: Option<usize> = None;

    for i in 0..args.len() {
        let arg = &args[i];

        if arg.value.variadic {
            if variadic_index.is_some() {
                return Err(PengError::new_positioned_message(
                    "cannot use more than one variadic unpacking in the same function call"
                        .to_string(),
                    arg.position.clone(),
                ));
            }

            if i + 1 != args.len() {
                return Err(PengError::new_positioned_message(
                    "variadic unpacking must be the last argument".to_string(),
                    arg.position.clone(),
                ));
            }

            variadic_index = Some(i);
        }

        match generate_expression(env, context, &arg.value.expression) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating function call arg".to_string(),
                )));
            }
        }
    }

    Ok(variadic_index)
}
