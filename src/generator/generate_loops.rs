use crate::core::*;
use crate::parser::*;
use crate::generator::*;

pub fn generate_while_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengWhileStatement,
) -> Result<(), PengError> {
    let condition_start = context.bytecode.len();

    match generate_expression(
        env,
        context,
        &statement.condition,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let end_jump = context.emit_jump_if_false();
    context.push_loop(Some(condition_start));

    let body_result = generate_scoped_statements(
        env,
        context,
        &statement.body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            return Err(e);
        }
    }

    context.bytecode.push(
        PengInstruction::Jump(condition_start),
    );

    let end = context.bytecode.len();
    context.patch_jump(end_jump, end);

    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            return Err(PengError::new_positioned_message(
                "missing while loop generation context".to_string(),
                statement.condition.position.clone(),
            ));
        }
    };

    context.patch_loop(
        loop_context,
        end,
        condition_start,
    );

    Ok(())
}

pub fn generate_loop_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    body: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    let loop_start = context.bytecode.len();
    context.push_loop(Some(loop_start));

    let body_result = generate_scoped_statements(
        env,
        context,
        body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            return Err(e);
        }
    }

    context.bytecode.push(
        PengInstruction::Jump(loop_start),
    );

    let end = context.bytecode.len();
    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            return Err(PengError::new_message(
                "missing loop generation context".to_string(),
            ));
        }
    };

    context.patch_loop(loop_context, end, loop_start);
    Ok(())
}

pub fn generate_for_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengForStatement,
) -> Result<(), PengError> {
    context.push_scope();

    match &statement.initializer {
        Some(initializer) => {
            match generate_statement(
                env,
                context,
                initializer,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_scope();
                    return Err(e);
                }
            }
        }
        None => {}
    }

    let condition_start = context.bytecode.len();
    let end_jump = match &statement.condition {
        Some(condition) => {
            match generate_expression(
                env,
                context,
                condition,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_scope();
                    return Err(e);
                }
            }

            Some(context.emit_jump_if_false())
        }
        None => None,
    };

    context.push_loop(None);

    let body_result = generate_scoped_statements(
        env,
        context,
        &statement.body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            context.pop_scope();
            return Err(e);
        }
    }

    let increment_start = context.bytecode.len();

    match &statement.increment {
        Some(increment) => {
            match generate_statement(
                env,
                context,
                increment,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_loop();
                    context.pop_scope();
                    return Err(e);
                }
            }
        }
        None => {}
    }

    context.bytecode.push(
        PengInstruction::Jump(condition_start),
    );

    let end = context.bytecode.len();

    match end_jump {
        Some(end_jump) => {
            context.patch_jump(end_jump, end);
        }
        None => {}
    }

    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            context.pop_scope();
            return Err(PengError::new_message(
                "missing for loop generation context".to_string(),
            ));
        }
    };

    context.patch_loop(
        loop_context,
        end,
        increment_start,
    );
    context.pop_scope();

    Ok(())
}
