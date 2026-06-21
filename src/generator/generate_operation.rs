use crate::core::*;
use crate::parser::*;
use crate::generator::*;


pub fn generate_operation_declaration_value(
    env: &PengEnv,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<PengValue, PengError> {
    generate_operation_value(
        env,
        &declaration.value.params,
        &declaration.value.body,
    )
}

pub fn generate_operation_value(
    env: &PengEnv,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengValue, PengError> {
    let mut context = PengGeneratorContext::new();

    for param in params {
        context.create_local(
            param.value.name.value.clone(),
        );
    }

    match generate_statements(env, &mut context, body) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.push_const_and_const_instruction(PengValue::Nil);
    context.bytecode.push(PengInstruction::Return);

    Ok(PengValue::Operation(
        PengOperation::Bytecode(
            PengBytecodeOperation {
                bytecode: context.bytecode,
                consts: context.consts,
                generics_count: 0,
                using_values: Vec::new(),
            },
        ),
    ))
}

pub fn generate_operation_call(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    left: &PengPositionedExpression,
    operation: &PengPositionedExpression,
    right: &PengPositionedExpression,
) -> Result<(), PengError> {
    match generate_expression(env, context, operation) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, context, left) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, context, right) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::OperationCall);

    Ok(())
}


pub fn generate_local_operation_declaration(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value = match generate_operation_declaration_value(
        env,
        declaration,
    ) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.push_const_and_const_instruction(value);

    let local = context.create_local(
        declaration.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}
