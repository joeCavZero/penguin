use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn create_anonymous_bytecode_function(
    env: &mut PengEnv,
    context: PengGeneratorContext,
    params: PengBytecodeFunctionParams,
) -> PengHeapPtr {
    let function = create_bytecode_function_value(context, params);

    env.create_heap_value(function)
}

pub fn create_bytecode_function_value(
    mut context: PengGeneratorContext,
    params: PengBytecodeFunctionParams,
) -> PengHeapValue {
    context.push_const_and_const_instruction(PengValue::Cell(PengCell::Nil));
    context.bytecode.push(PengInstruction::Return);

    PengHeapValue::Function(PengFunction::Bytecode(PengBytecodeFunction {
        bytecode: context.bytecode,
        consts: context.consts,
        using_values: Vec::new(),
        params,
    }))
}

pub fn generate_function_declaration_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<PengHeapValue, PengError> {
    generate_function_value(
        env,
        globals,
        &declaration.value.params,
        &declaration.value.body,
    )
}

pub fn generate_function_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengHeapValue, PengError> {
    let mut context = PengGeneratorContext::new();

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

    match generate_statements(env, globals, &mut context, body) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_function".to_string(),
            )));
        }
    }

    Ok(create_bytecode_function_value(context, function_params))
}

pub fn generate_function_call(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    call: &PengFuncCallExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &call.function) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_function".to_string(),
            )));
        }
    }

    for arg in &call.args {
        match generate_expression(env, globals, context, arg) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_function".to_string(),
                )));
            }
        }
    }

    context
        .bytecode
        .push(PengInstruction::FunctionCall(call.args.len()));

    Ok(())
}

pub fn generate_local_function_declaration(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let global = get_allocated_global(env, globals, &declaration.value.name.value);

    match global {
        Some(value_ptr) => {
            let value = match generate_function_declaration_value(env, globals, declaration) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_function".to_string(),
                    )));
                }
            };

            context
                .bytecode
                .push(PengInstruction::PushHeapRef(value_ptr));

            context.push_const_and_const_instruction(PengValue::Heap(value));

            context.bytecode.push(PengInstruction::StoreHeap);

            Ok(())
        }

        None => {
            let value = match generate_function_declaration_value(env, globals, declaration) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_function".to_string(),
                    )));
                }
            };

            context.push_const_and_const_instruction(PengValue::Heap(value));

            let local = context.create_local(declaration.value.name.value.clone());

            context.bytecode.push(PengInstruction::StoreLocal(local));

            Ok(())
        }
    }
}
