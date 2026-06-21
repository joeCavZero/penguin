use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_program(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<(HashMap<PengNamePoolPtr, PengValuePtr>, PengValuePtr), PengError> {
    let declarations = match ast {
        PengAST::Program(declarations) => declarations,
        PengAST::Script(_) => {
            return Err(PengError::new_message(
                "expected program AST".to_string(),
            ));
        }
    };

    let mut globals = match allocate_program_globals(env, declarations) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let mut context = PengGeneratorContext::new();

    match generate_program_initialization(
        env,
        &mut globals,
        declarations,
        &mut context,
    ) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };

    let program_init = create_anonymous_bytecode_function(env, context);

    Ok((globals, program_init))
}

fn allocate_program_globals(
    env: &mut PengEnv,
    declarations: &Vec<PengDeclaration>,
) -> Result<HashMap<PengNamePoolPtr, PengValuePtr>, PengError> {
    let mut globals = HashMap::new();

    for declaration in declarations {
        let name = declaration_name(declaration);
        let name_ptr = env.get_pooled_name(name);

        if globals.contains_key(&name_ptr) {
            return Err(PengError::new_message(
                "duplicated global declaration".to_string(),
            ));
        }

        let value_ptr = env.create_value(PengValue::Nil);
        globals.insert(name_ptr, value_ptr);
    }

    Ok(globals)
}

pub fn get_allocated_global(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    name: &str,
) -> Option<PengValuePtr> {
    let name_ptr = env.get_pooled_name(name.to_string());
    globals.get(&name_ptr).copied()
}

fn declaration_name(declaration: &PengDeclaration) -> String {
    match declaration {
        PengDeclaration::Variable(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
    }
}

fn generate_program_initialization(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    declarations: &Vec<PengDeclaration>,
    context: &mut PengGeneratorContext,
) -> Result<(), PengError> {
    for declaration in declarations {
        match declaration {
            PengDeclaration::Function(function) => {
                match generate_global_function(env, globals, context, function) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Operation(operation) => {
                match generate_global_operation(env, globals, context, operation) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Type(declaration) => {
                if declaration.value.value.is_some() {
                    match generate_global_type_value(env, globals, context, declaration) {
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
                match generate_global_variable(env, globals, context, variable) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::As(declaration) => {
                match generate_global_as_declaration(env, globals, context, declaration) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            PengDeclaration::Function(_) => {}
            PengDeclaration::Type(type_declaration) => {
                if type_declaration.value.value.is_none() {
                    match generate_global_structured_type(env, globals, context, type_declaration) {
                        Ok(()) => {},
                        Err(e) => return Err(e),
                    };
                }
            }
            PengDeclaration::Module(declaration) => {
                return Err(PengError::new_positioned_message(
                    "global module declarations require module-construction bytecode support"
                        .to_string(),
                    declaration.position.clone(),
                ));
            }
            PengDeclaration::Operation(_) => {}
        }
    }

    Ok(())
}

fn generate_global_structured_type(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match env
        .get_global(&declaration.value.name.value)
        .ok_or_else(|| {
            PengError::new_positioned_message(
                "global type was not allocated".to_string(),
                declaration.value.name.position.clone(),
            )
        }) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

    let literal = PengTypeLiteral {
        generics: declaration.value.generics.clone(),
        supers: declaration.value.supers.clone(),
        fields: declaration.value.fields.clone(),
        functions: declaration.value.functions.clone(),
    };

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));
    match generate_type_literal(env, globals, context, &literal) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };
    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}

fn generate_global_type_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, globals, &declaration.value.name.value) {
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

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));

    match generate_type_expression(env, globals, context, value) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}

fn generate_global_operation(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, globals, &declaration.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global operation was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_operation_declaration_value(env, globals, declaration) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreValue);

    Ok(())
}

fn generate_global_function(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, globals, &declaration.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global function was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_function_declaration_value(env, globals, declaration) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreValue);

    Ok(())
}

fn generate_global_as_declaration(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, globals, &declaration.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global as declaration was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));

    match generate_expression(env, globals, context, &declaration.value.value) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}

fn generate_global_variable(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    variable: &PengPositionedVariableDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, globals, &variable.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global variable was not allocated".to_string(),
                variable.value.name.position.clone(),
            ));
        }
    };

    context
        .bytecode
        .push(PengInstruction::PushValueRef(value_ptr));

    match &variable.value.value {
        Some(value) => match generate_expression(env, globals, context, value) {
            Ok(()) => {}
            Err(e) => return Err(e),
        },
        None => {
            context.push_const_and_const_instruction(PengValue::Nil);
        }
    }

    context.bytecode.push(PengInstruction::StoreValue);
    Ok(())
}
