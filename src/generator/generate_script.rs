use crate::parser::*;
use crate::core::*;
use crate::generator::generator_utils::{
    create_anonymous_bytecode_function,
    generate_expression,
    PengGeneratorContext,
};

pub fn generate_script(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengValuePtr, PengError> {
    let statements = match ast {
        PengAST::Script(statements) => statements,
        PengAST::Program(_) => {
            return Err(PengError::new_message(
                "expected script AST".to_string(),
            ));
        }
    };

    let mut context = PengGeneratorContext::new();

    match generate_script_statements(
        env,
        statements,
        &mut context,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let script_function = create_anonymous_bytecode_function(
        env,
        context,
    );

    Ok(script_function)
}

fn generate_script_statements(
    env: &PengEnv,
    statements: &Vec<PengPositionedStatement>,
    context: &mut PengGeneratorContext,
) -> Result<(), PengError> {
    for statement in statements {
        match &statement.value {
            PengStatement::Declaration(
                PengDeclaration::Variable(variable),
            ) => {
                match generate_local_variable(
                    env,
                    context,
                    variable,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengStatement::Declaration(
                PengDeclaration::As(declaration),
            ) => {
                match generate_local_as_declaration(
                    env,
                    context,
                    declaration,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengStatement::Expression(expression) => {
                match generate_expression(
                    env,
                    context,
                    expression,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }

                context.bytecode.push(PengInstruction::Pop(1));
            }
            PengStatement::Declaration(_) => {
                todo!("generate non-variable script declaration")
            }
            PengStatement::Block(_) => {
                todo!("generate block statement")
            }
            PengStatement::Return(_) => {
                todo!("generate return statement")
            }
            PengStatement::If(_) => {
                todo!("generate if statement")
            }
            PengStatement::While(_) => {
                todo!("generate while statement")
            }
            PengStatement::For(_) => {
                todo!("generate for statement")
            }
            PengStatement::ForEach(_) => {
                todo!("generate foreach statement")
            }
            PengStatement::Loop(_) => {
                todo!("generate loop statement")
            }
            PengStatement::Break => {
                todo!("generate break statement")
            }
            PengStatement::Continue => {
                todo!("generate continue statement")
            }
            PengStatement::Assign(_) => {
                todo!("generate assignment statement")
            }
        }
    }

    Ok(())
}

fn generate_local_as_declaration(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
) -> Result<(), PengError> {
    match generate_expression(
        env,
        context,
        &declaration.value.value,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let local = context.create_local(
        declaration.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}

fn generate_local_variable(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    variable: &PengPositionedVariableDeclaration,
) -> Result<(), PengError> {
    match &variable.value.value {
        Some(value) => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
        None => {
            context.push_const(PengValue::Nil);
        }
    }

    let local = context.create_local(
        variable.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}
