use crate::core::*;
use crate::parser::*;
use crate::generator::*;


pub fn generate_statements(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    for statement in statements {
        match generate_statement(env, context, statement) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

pub fn generate_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengPositionedStatement,
) -> Result<(), PengError> {
    match &statement.value {
        PengStatement::Declaration(
            PengDeclaration::Variable(variable),
        ) => {
            generate_local_variable(env, context, variable)
        }
        PengStatement::Declaration(
            PengDeclaration::As(declaration),
        ) => {
            generate_local_as_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(
            PengDeclaration::Function(declaration),
        ) => {
            generate_local_function_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(
            PengDeclaration::Operation(declaration),
        ) => {
            generate_local_operation_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(
            PengDeclaration::Type(declaration),
        ) => {
            generate_local_type_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(_) => {
            todo!("generate non-function local declaration")
        }
        PengStatement::Block(statements) => {
            context.push_scope();

            let result = generate_statements(
                env,
                context,
                statements,
            );

            context.pop_scope();

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        }
        PengStatement::Return(value) => {
            match value {
                Some(value) => {
                    match generate_expression(
                        env,
                        context,
                        value,
                    ) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }
                }
                None => {
                    context.push_const_and_const_instruction(PengValue::Nil);
                }
            }

            context.bytecode.push(PengInstruction::Return);
            Ok(())
        }
        PengStatement::Assign(assignment) => {
            generate_assignment(env, context, assignment)
        }
        PengStatement::Expression(expression) => {
            match generate_expression(env, context, expression) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            context.bytecode.push(PengInstruction::Pop(1));
            Ok(())
        }
        PengStatement::If(if_statement) => {
            generate_if_statement(
                env,
                context,
                if_statement,
            )
        }
        PengStatement::While(while_statement) => {
            generate_while_statement(
                env,
                context,
                while_statement,
            )
        }
        PengStatement::For(for_statement) => {
            generate_for_statement(
                env,
                context,
                for_statement,
            )
        }
        PengStatement::Loop(body) => {
            generate_loop_statement(env, context, body)
        }
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
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    context.push_scope();

    let result = generate_statements(
        env,
        context,
        statements,
    );

    context.pop_scope();

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_if_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengIfStatement,
) -> Result<(), PengError> {
    match generate_expression(
        env,
        context,
        &statement.condition,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let false_jump = context.emit_jump_if_false();

    match generate_scoped_statements(
        env,
        context,
        &statement.then_branch,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match &statement.else_branch {
        Some(else_branch) => {
            let end_jump = context.emit_jump();
            let else_start = context.bytecode.len();
            context.patch_jump(false_jump, else_start);

            match generate_scoped_statements(
                env,
                context,
                else_branch,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
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
