use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_operation_declaration_value(
    env: &mut PengEnv,

    declaration: &PengPositionedOperationDeclaration,
) -> Result<PengBox, PengError> {
    generate_operation_value(
        env,
        &declaration.value.params,
        &declaration.value.body,
    )
}

pub fn generate_operation_value(
    env: &mut PengEnv,

    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengBox, PengError> {
    let mut context = PengGeneratorContext::new();

    for param in params {
        context.create_local(param.value.name.value.clone());
    }

    match generate_statements(env, &mut context, body) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    }

    context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil));
    context.bytecode.push(PengInstruction::Return);

    Ok(PengBox::Operation(PengOperation::Bytecode(
        PengBytecodeOperation {
            bytecode: context.bytecode,
            consts: context.consts,
            using_values: Vec::new(),
        },
    )))
}

pub fn generate_operation_call(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    left: &PengPositionedExpression,
    operation: &PengPositionedExpression,
    right: &PengPositionedExpression,
) -> Result<(), PengError> {
    match generate_expression(env, context, operation) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    }

    match generate_expression(env, context, left) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    }

    match generate_expression(env, context, right) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    }

    context.bytecode.push(PengInstruction::OperationCall);

    Ok(())
}

pub fn generate_local_operation_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    let value = match generate_operation_declaration_value(env, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    };

    context.push_const_and_const_instruction(env, PengValue::Box(value));
    generate_make_immutable_if_needed(context, immutable);
    let local = context.create_local(declaration.value.name.value.clone());
    context.bytecode.push(PengInstruction::StoreLocal(local));

    Ok(())
}
