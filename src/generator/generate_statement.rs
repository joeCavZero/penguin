use crate::core::*;
pub use crate::generator::generate::*;

use crate::parser::*;

pub fn generate_statements(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    for statement in statements {
        match generate_statement(env, context, statement) {
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

    context: &mut PengGeneratorContext,
    statement: &PengPositionedStatement,
) -> Result<(), PengError> {
    match &statement.value {
        PengStatement::Declaration(binded_declaration) => match binded_declaration {
            PengBinded::Mutable(declaration) => {
                generate_binded_local_declaration(env, context, declaration, false)
            }

            PengBinded::Immutable(declaration) => {
                generate_binded_local_declaration(env, context, declaration, true)
            }
        },
        PengStatement::Block(statements) => {
            context.push_scope();

            let result = generate_statements(env, context, statements);

            context.pop_scope();

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        }

        PengStatement::Return(value) => {
            match value {
                Some(value) => match generate_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_statement".to_string(),
                        )));
                    }
                },
                None => {
                    context.push_const_and_const_instruction(
                        env,
                        PengValue::Cell(PengCell::Nil),
                        statement.position.clone(),
                    );
                }
            }

            context
                .push_positioned_instruction(PengInstruction::Return, statement.position.clone());
            Ok(())
        }

        PengStatement::Assign(assignment) => generate_assignment(env, context, assignment),

        PengStatement::Expression(expression) => {
            match generate_expression(env, context, expression) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_statement".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(PengInstruction::Pop, statement.position.clone());
            Ok(())
        }

        PengStatement::If(if_statement) => {
            generate_if_statement(env, context, if_statement, statement.position.clone())
        }

        PengStatement::Match(match_statement) => {
            generate_match_statement(env, context, match_statement, statement.position.clone())
        }

        PengStatement::While(while_statement) => {
            generate_while_statement(env, context, while_statement, statement.position.clone())
        }

        PengStatement::For(for_statement) => {
            generate_for_statement(env, context, for_statement, statement.position.clone())
        }

        PengStatement::Loop(body) => {
            generate_loop_statement(env, context, body, statement.position.clone())
        }

        PengStatement::Break => {
            if context.emit_break(statement.position.clone()) {
                Ok(())
            } else {
                Err(PengError::new_positioned_message(
                    "'break' used outside a loop".to_string(),
                    statement.position.clone(),
                ))
            }
        }

        PengStatement::Continue => {
            if context.emit_continue(statement.position.clone()) {
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

    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    context.push_scope();

    let result = generate_statements(env, context, statements);

    context.pop_scope();

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_if_statement(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    statement: &PengIfStatement,
    pos: PengPosition,
) -> Result<(), PengError> {
    match generate_expression(env, context, &statement.condition) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    let false_jump = context.emit_jump_if_false(pos.clone());

    match generate_scoped_statements(env, context, &statement.then_branch) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    match &statement.else_branch {
        Some(else_branch) => {
            let end_jump = context.emit_jump(pos.clone());
            let else_start = context.bytecode.len();
            context.patch_jump(false_jump, else_start);

            match generate_scoped_statements(env, context, else_branch) {
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

    context: &mut PengGeneratorContext,
    statement: &PengMatchStatement,
    pos: PengPosition,
) -> Result<(), PengError> {
    let matched_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &statement.value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_statement".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreLocal(matched_local), pos.clone());

    let mut end_jumps = Vec::new();

    for arm in &statement.arms {
        context.push_positioned_instruction(
            PengInstruction::PushLocal(matched_local),
            arm.pattern.position.clone(),
        );

        match generate_expression(env, context, &arm.pattern) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        }

        context.push_positioned_instruction(PengInstruction::Equals, arm.pattern.position.clone());

        let next_arm_jump = context.emit_jump_if_false(pos.clone());

        match generate_scoped_statements(env, context, &arm.body) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_statement".to_string(),
                )));
            }
        }

        let end_jump = context.emit_jump(pos.clone());
        end_jumps.push(end_jump);

        let next_arm = context.bytecode.len();
        context.patch_jump(next_arm_jump, next_arm);
    }

    match &statement.elsing {
        Some(body) => match generate_scoped_statements(env, context, body) {
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

pub fn generate_make_immutable_if_needed(
    context: &mut PengGeneratorContext,
    immutable: bool,
    pos: PengPosition,
) {
    if immutable {
        context.push_positioned_instruction(PengInstruction::MakeImmutable, pos);
    }
}

pub fn generate_binded_local_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    match declaration {
        PengDeclaration::Var(variable) => {
            generate_local_variable(env, context, variable, immutable)
        }

        PengDeclaration::As(declaration) => {
            generate_local_as_declaration(env, context, declaration, immutable)
        }

        PengDeclaration::Function(declaration) => {
            generate_local_function_declaration(env, context, declaration, immutable)
        }

        PengDeclaration::Operation(declaration) => {
            generate_local_operation_declaration(env, context, declaration, immutable)
        }

        PengDeclaration::Type(declaration) => {
            generate_local_type_declaration(env, context, declaration, immutable)
        }

        PengDeclaration::Module(declaration) => {
            generate_local_module_declaration(env, context, declaration, immutable)
        }
    }
}
