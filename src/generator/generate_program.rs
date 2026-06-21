use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_program(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengValuePtr, PengError> {
    let declarations = match ast {
        PengAST::Program(declarations) => declarations,
        PengAST::Script(_) => {
            return Err(PengError::new_message(
                "expected program AST".to_string(),
            ));
        }
    };

    match allocate_program_globals(env, declarations) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let mut context = PengGeneratorContext::new();

    match generate_program_initialization(
        env,
        declarations,
        &mut context,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let program_init = create_anonymous_bytecode_function(
        env,
        context,
    );

    Ok(program_init)
}

fn allocate_program_globals(
    env: &mut PengEnv,
    declarations: &Vec<PengDeclaration>,
) -> Result<(), PengError> {
    for declaration in declarations {
        let name = declaration_name(declaration);
        env.create_global(name);
    }

    Ok(())
}

fn declaration_name(
    declaration: &PengDeclaration,
) -> String {
    match declaration {
        PengDeclaration::Variable(declaration) => {
            declaration.value.name.value.clone()
        }
        PengDeclaration::As(declaration) => {
            declaration.value.name.value.clone()
        }
        PengDeclaration::Function(declaration) => {
            declaration.value.name.value.clone()
        }
        PengDeclaration::Type(declaration) => {
            declaration.value.name.value.clone()
        }
        PengDeclaration::Module(declaration) => {
            declaration.value.name.value.clone()
        }
        PengDeclaration::Operation(declaration) => {
            declaration.value.name.value.clone()
        }
    }
}

fn generate_program_initialization(
    env: &PengEnv,
    declarations: &Vec<PengDeclaration>,
    context: &mut PengGeneratorContext,
) -> Result<(), PengError> {
    for declaration in declarations {
        match declaration {
            PengDeclaration::Function(function) => {
                match generate_global_function(
                    env,
                    context,
                    function,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Operation(operation) => {
                match generate_global_operation(
                    env,
                    context,
                    operation,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Type(declaration) => {
                if declaration.value.value.is_some() {
                    match generate_global_type_value(
                        env,
                        context,
                        declaration,
                    ) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }
                }
            }
            _ => {}
        }
    }

    for declaration in declarations {
        match declaration {
            PengDeclaration::Variable(variable) => {
                match generate_global_variable(
                    env,
                    context,
                    variable,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::As(declaration) => {
                match generate_global_as_declaration(
                    env,
                    context,
                    declaration,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Function(_) => {}
            PengDeclaration::Type(type_declaration) => {
                if type_declaration.value.value.is_none() {
                    todo!("generate structured global type declaration")
                }
            }
            PengDeclaration::Module(_) => {
                todo!("generate global module declaration")
            }
            PengDeclaration::Operation(_) => {}
        }
    }

    Ok(())
}

fn generate_global_type_value(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env.get_global(
        &declaration.value.name.value,
    ) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global type value was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match &declaration.value.value {
        Some(value) => value,
        None => {
            return Err(PengError::new_positioned_message(
                "expected type declaration value".to_string(),
                declaration.position.clone(),
            ));
        }
    };

    context.bytecode.push(
        PengInstruction::PushValueRef(value_ptr),
    );

    match generate_type_expression(env, context, value) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}

fn generate_global_operation(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env.get_global(
        &declaration.value.name.value,
    ) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global operation was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_operation_declaration_value(
        env,
        declaration,
    ) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.bytecode.push(
        PengInstruction::PushValueRef(value_ptr),
    );
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreValue);

    Ok(())
}

fn generate_global_function(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env.get_global(
        &declaration.value.name.value,
    ) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global function was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_function_declaration_value(
        env,
        declaration,
    ) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.bytecode.push(
        PengInstruction::PushValueRef(value_ptr),
    );
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreValue);

    Ok(())
}

fn generate_global_as_declaration(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env.get_global(
        &declaration.value.name.value,
    ) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global as declaration was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    context.bytecode.push(
        PengInstruction::PushValueRef(value_ptr),
    );

    match generate_expression(
        env,
        context,
        &declaration.value.value,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}

fn generate_global_variable(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    variable: &PengPositionedVariableDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env.get_global(&variable.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global variable was not allocated".to_string(),
                variable.value.name.position.clone(),
            ));
        }
    };

    context.bytecode.push(
        PengInstruction::PushValueRef(value_ptr),
    );

    match &variable.value.value {
        Some(value) => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
        None => {
            context.push_const_and_const_instruction(PengValue::Nil);
        }
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}
