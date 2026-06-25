use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_statements(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    for statement in statements {
        match generate_statement(env, globals, context, statement) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        }
    }

    Ok(())
}

pub fn generate_statement(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    statement: &PengPositionedStatement,
) -> Result<(), PengError> {
    match &statement.value {
        PengStatement::Declaration(binded_declaration) => {
            let declaration = match binded_declaration {
                PengBinded::Mutable(declaration) => declaration,
                PengBinded::Immutable(declaration) => declaration,
            };

            match declaration {
                PengDeclaration::Var(variable) => {
                    generate_local_variable(env, globals, context, variable)
                }
                PengDeclaration::As(declaration) => {
                    generate_local_as_declaration(env, globals, context, declaration)
                }
                PengDeclaration::Function(declaration) => {
                    generate_local_function_declaration(env, globals, context, declaration)
                }
                PengDeclaration::Operation(declaration) => {
                    generate_local_operation_declaration(env, globals, context, declaration)
                }
                PengDeclaration::Type(declaration) => {
                    generate_local_type_declaration(env, globals, context, declaration)
                }
                PengDeclaration::Module(declaration) => {
                    generate_local_module_declaration(env, globals, context, declaration)
                }
            }
        }

        PengStatement::Block(statements) => {
            context.push_scope();

            let result = generate_statements(env, globals, context, statements);

            context.pop_scope();

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        }

        PengStatement::Return(value) => {
            match value {
                Some(value) => match generate_expression(env, globals, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_statement".to_string(),
                        )));
                    }
                },
                None => {
                    context.push_const_and_const_instruction(PengValue::Cell(PengCell::Nil));
                }
            }

            context.bytecode.push(PengInstruction::Return);
            Ok(())
        }

        PengStatement::Assign(assignment) => generate_assignment(env, globals, context, assignment),

        PengStatement::Expression(expression) => {
            match generate_expression(env, globals, context, expression) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_statement".to_string(),
                    )));
                }
            }

            context.bytecode.push(PengInstruction::Pop);
            Ok(())
        }

        PengStatement::If(if_statement) => {
            generate_if_statement(env, globals, context, if_statement)
        }

        PengStatement::Match(match_statement) => {
            generate_match_statement(env, globals, context, match_statement)
        }

        PengStatement::While(while_statement) => {
            generate_while_statement(env, globals, context, while_statement)
        }

        PengStatement::For(for_statement) => {
            generate_for_statement(env, globals, context, for_statement)
        }

        PengStatement::Loop(body) => generate_loop_statement(env, globals, context, body),

        PengStatement::Break => {
            if context.emit_break() {
                Ok(())
            } else {
                Err(PengError::new_positioned_message(
                    "'break' used outside a loop".to_string(),
                    statement.position.clone(),
                ))
            }
        }

        PengStatement::Continue => {
            if context.emit_continue() {
                Ok(())
            } else {
                Err(PengError::new_positioned_message(
                    "'continue' used outside a loop".to_string(),
                    statement.position.clone(),
                ))
            }
        }
    }
}

pub fn generate_scoped_statements(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    context.push_scope();

    let result = generate_statements(env, globals, context, statements);

    context.pop_scope();

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_if_statement(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    statement: &PengIfStatement,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &statement.condition) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    let false_jump = context.emit_jump_if_false();

    match generate_scoped_statements(env, globals, context, &statement.then_branch) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    match &statement.else_branch {
        Some(else_branch) => {
            let end_jump = context.emit_jump();
            let else_start = context.bytecode.len();
            context.patch_jump(false_jump, else_start);

            match generate_scoped_statements(env, globals, context, else_branch) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_statement".to_string(),
                    )));
                }
            }

            let end = context.bytecode.len();
            context.patch_jump(end_jump, end);
        }
        None => {
            let end = context.bytecode.len();
            context.patch_jump(false_jump, end);
        }
    }

    Ok(())
}

pub fn generate_match_statement(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    statement: &PengMatchStatement,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &statement.value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    let matched_local = context.create_temporary_local();
    context
        .bytecode
        .push(PengInstruction::StoreLocal(matched_local));

    let mut end_jumps = Vec::new();

    for arm in &statement.arms {
        context
            .bytecode
            .push(PengInstruction::PushLocal(matched_local));

        match generate_expression(env, globals, context, &arm.pattern) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        }

        context.bytecode.push(PengInstruction::Equals);

        let next_arm_jump = context.emit_jump_if_false();

        match generate_scoped_statements(env, globals, context, &arm.body) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        }

        let end_jump = context.emit_jump();
        end_jumps.push(end_jump);

        let next_arm = context.bytecode.len();
        context.patch_jump(next_arm_jump, next_arm);
    }

    match &statement.elsing {
        Some(body) => match generate_scoped_statements(env, globals, context, body) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        },
        None => {}
    }

    let end = context.bytecode.len();

    for jump in end_jumps {
        context.patch_jump(jump, end);
    }

    Ok(())
}
