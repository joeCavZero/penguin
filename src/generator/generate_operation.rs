use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_operation_declaration_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<PengHeapValue, PengError> {
    generate_operation_value(
        env,
        globals,
        &declaration.value.params,
        &declaration.value.body,
    )
}

pub fn generate_operation_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengHeapValue, PengError> {
    let mut context = PengGeneratorContext::new();

    for param in params {
        context.create_local(param.value.name.value.clone());
    }

    match generate_statements(env, globals, &mut context, body) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.push_const_and_const_instruction(PengValue::Cell(PengCell::Nil));
    context.bytecode.push(PengInstruction::Return);

    Ok(PengHeapValue::Operation(PengOperation::Bytecode(
        PengBytecodeOperation {
            bytecode: context.bytecode,
            consts: context.consts,
            using_values: Vec::new(),
        },
    )))
}

pub fn generate_operation_call(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
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
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value = match generate_operation_declaration_value(env, globals, declaration) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.push_const_and_const_instruction(PengValue::Heap(value));

    let local = context.create_local(declaration.value.name.value.clone());
    context.bytecode.push(PengInstruction::StoreLocal(local));

    Ok(())
}
