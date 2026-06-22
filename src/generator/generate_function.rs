use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn create_anonymous_bytecode_function(
    env: &mut PengEnv,
    context: PengGeneratorContext,
) -> PengValuePtr {
    let function = create_bytecode_function_value(context, 0);

    env.create_value(function)
}

pub fn create_bytecode_function_value(
    mut context: PengGeneratorContext,
    generics_count: usize,
) -> PengValue {
    context.push_const_and_const_instruction(PengValue::Nil);
    context.bytecode.push(PengInstruction::Return);

    PengValue::Function(PengFunction::Bytecode(PengBytecodeFunction {
        bytecode: context.bytecode,
        consts: context.consts,
        generics_count,
        using_values: Vec::new(),
    }))
}

pub fn generate_function_declaration_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<PengValue, PengError> {
    generate_function_value(
        env,
        globals,
        &declaration.value.generics,
        &declaration.value.params,
        &declaration.value.body,
    )
}

pub fn generate_function_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    generics: &Vec<PengPositioned<String>>,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengValue, PengError> {
    let mut context = PengGeneratorContext::new();

    for param in params {
        context.create_local(param.value.name.value.clone());
    }

    match generate_statements(env, globals, &mut context, body) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(create_bytecode_function_value(context, generics.len()))
}

pub fn generate_function_call(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    call: &PengFuncCallExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &call.function) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    for generic in &call.generics {
        match generate_expression(env, globals, context, generic) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    for arg in &call.args {
        match generate_expression(env, globals, context, arg) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    context.bytecode.push(PengInstruction::FunctionCall {
        generics: call.generics.len(),
        params: call.args.len(),
    });

    Ok(())
}

pub fn generate_local_function_declaration(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value = match generate_function_declaration_value(env, globals, declaration) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.push_const_and_const_instruction(value);

    let local = context.create_local(declaration.value.name.value.clone());
    context.bytecode.push(PengInstruction::StoreLocal(local));

    Ok(())
}
