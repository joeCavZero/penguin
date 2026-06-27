use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_program(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
) -> Result<PengHeapPtr, PengError> {
    match allocate_program_globals(env, declarations) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    }

    let mut context = PengGeneratorContext::new();

    generate_program_initialization(env, declarations, &mut context)?;

    let program_init = create_anonymous_bytecode_function(
        env,
        context,
        PengBytecodeFunctionParams::Fixed(0),
    );

    Ok(program_init)
}

fn allocate_program_globals(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
) -> Result<(), PengError> {
    for declaration in declarations {
        let name = declaration_name(declaration);
        let name_ptr = env.ensure_pooled_name_ptr(name);

        let heap_ptr = env.create_heap_value(
            PengHeapValue::Object(PengObject::new_empty()),
        );

        let value = match declaration {
            PengBinded::Mutable(_) => {
                PengBinded::Mutable(PengCell::Reference(heap_ptr))
            }

            PengBinded::Immutable(_) => {
                PengBinded::Immutable(PengCell::Reference(heap_ptr))
            }
        };

        env.create_global(name_ptr, value)?;
    }

    Ok(())
}

pub fn get_allocated_global(
    env: &mut PengEnv,
    name: &str,
) -> Option<PengHeapPtr> {
    match env.get_global_by_name_str(name) {
        Some(cell) => match cell.value() {
            PengCell::Reference(ptr) => Some(*ptr),
            _ => None,
        },
        None => None,
    }
}

fn declaration_name(declaration: &PengBindedDeclaration) -> String {
    let declaration = match declaration {
        PengBinded::Mutable(declaration) => declaration,
        PengBinded::Immutable(declaration) => declaration,
    };

    match declaration {
        PengDeclaration::Var(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
        PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
    }
}

fn generate_program_initialization(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
    context: &mut PengGeneratorContext,
) -> Result<(), PengError> {
    for declaration in declarations {
        let declaration = match declaration {
            PengBinded::Mutable(declaration) => declaration,
            PengBinded::Immutable(declaration) => declaration,
        };

        match declaration {
            PengDeclaration::Function(function) => {
                match generate_global_function(env, context, function) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_program".to_string(),
                        )));
                    }
                }
            }

            PengDeclaration::Operation(operation) => {
                match generate_global_operation(env, context, operation) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_program".to_string(),
                        )));
                    }
                }
            }

            PengDeclaration::Type(declaration) => {
                if declaration.value.value.is_some() {
                    match generate_global_type_value(env, context, declaration) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_program".to_string(),
                            )));
                        }
                    }
                }
            }

            _ => {}
        }
    }

    Ok(())
}

fn generate_global_type_value(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, &declaration.value.name.value) {
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
        .push(PengInstruction::PushHeapRef(value_ptr));

    match generate_type_expression(env, context, value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    }

    context.bytecode.push(PengInstruction::StoreHeap);
    Ok(())
}

fn generate_global_operation(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, &declaration.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global operation was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_operation_declaration_value(env, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    };

    context
        .bytecode
        .push(PengInstruction::PushHeapRef(value_ptr));
    context.push_const_and_const_instruction(env, PengValue::Heap(value));
    context.bytecode.push(PengInstruction::StoreHeap);

    Ok(())
}

fn generate_global_function(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, &declaration.value.name.value) {
        Some(value_ptr) => value_ptr,
        None => {
            return Err(PengError::new_positioned_message(
                "global function was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_function_declaration_value(env, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    };

    context
        .bytecode
        .push(PengInstruction::PushHeapRef(value_ptr));
    context.push_const_and_const_instruction(env, PengValue::Heap(value));
    context.bytecode.push(PengInstruction::StoreHeap);

    Ok(())
}
