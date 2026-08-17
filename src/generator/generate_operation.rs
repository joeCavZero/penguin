use crate::core::*;
pub use crate::generator::generate::*;

use crate::parser::*;

pub fn generate_operation_declaration_value(
    env: &mut PengEnv,
    context: &PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<PengBox, PengError> {
    generate_operation_value(
        env,
        context,
        &declaration.value.params,
        &declaration.value.body,
        declaration.position.clone(),
    )
}

pub fn generate_operation_value(
    env: &mut PengEnv,
    parent_context: &PengGeneratorContext,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
    pos: PengPosition,
) -> Result<PengBox, PengError> {
    let mut context = parent_context.new_child_context();

    for param in params {
        context.create_local(param.value.name.value.clone());
    }
    context.mark_frame_prefix_locals();

    match generate_statements(env, &mut context, body) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
            )));
        }
    }

    context.prepend_frame_local_reserves(pos.clone());
    context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil), pos.clone());
    context.push_positioned_instruction(PengInstruction::Return, pos);
    debug_assert_eq!(context.bytecode.len(), context.positions.len());

    Ok(PengBox::Operation(PengOperation::Bytecode(
        PengBytecodeOperation {
            bytecode: context.bytecode,
            positions: context.positions,
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
    pos: PengPosition,
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

    context.push_positioned_instruction(PengInstruction::OperationCall, pos);

    Ok(())
}

pub fn generate_local_operation_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    let value = match generate_operation_declaration_value(env, context, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_operation".to_string(),
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
