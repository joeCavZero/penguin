use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_operation_declaration_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<PengValue, PengError> {
    generate_operation_value(env, globals, &declaration.value.params, &declaration.value.body)
}

pub fn generate_operation_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
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

    //context.push_const_and_const_instruction(PengValue::Nil);
    //context.bytecode.push(PengInstruction::Return);

    Ok(PengValue::Operation(PengOperation::Bytecode(
        PengBytecodeOperation {
            bytecode: context.bytecode,
            consts: context.consts,
            generics_count: 0,
            using_values: Vec::new(),
        },
    )))
}

pub fn generate_operation_call(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    left: &PengPositionedExpression,
    operation: &PengPositionedExpression,
    right: &PengPositionedExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, operation) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, globals, context, left) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, globals, context, right) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::OperationCall);

    Ok(())
}

pub fn generate_local_operation_declaration(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value = match generate_operation_declaration_value(env, globals, declaration) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.push_const_and_const_instruction(value);

    let local = context.create_local(declaration.value.name.value.clone());
    context.bytecode.push(PengInstruction::StoreLocal(local));

    Ok(())
}
