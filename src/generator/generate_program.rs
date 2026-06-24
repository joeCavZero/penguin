use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_program(
    env: &mut PengEnv,
    declarations: &Vec<PengBinded<PengDeclaration>>,
    globals: &HashMap<usize, usize>,
) -> Result<(HashMap<PengNamePoolPtr, PengHeapPtr>, PengHeapPtr), PengError> {
    let local_globals = match allocate_program_globals(env, declarations) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut full_globals = globals.clone();
    full_globals.extend(local_globals);

    let mut context = PengGeneratorContext::new();

    match generate_program_initialization(
        env,
        &mut full_globals,
        declarations,
        &mut context,
    ) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };

    let program_init = create_anonymous_bytecode_function(env, context);

    Ok((full_globals, program_init))
}

fn allocate_program_globals(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
) -> Result<HashMap<PengNamePoolPtr, PengHeapPtr>, PengError> {
    let mut globals = HashMap::new();

    for declaration in declarations {
        let name = declaration_name(declaration);
        let name_ptr = env.ensure_pooled_name_ptr(name);

        if globals.contains_key(&name_ptr) {
            return Err(PengError::new_message(
                "duplicated global declaration".to_string(),
            ));
        }

        let value_ptr = match declaration {
            PengBinded::Mutable(_) => {
                env.create_binded_stated_heap(PengBinded::Mutable(PengStated::Initialized(PengValue::Nil)))
            }

            PengBinded::Immutable(_) => {
                env.create_binded_stated_heap(PengBinded::Immutable(PengStated::Uninitialized))
            }
        };

        globals.insert(name_ptr, value_ptr);
    }

    Ok(globals)
}

pub fn allocate_script_globals(
    env: &mut PengEnv,
    statements: &Vec<PengPositioned<PengStatement>>,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
) -> Result<(), PengError> {
    for statement in statements {
        match &statement.value {
            PengStatement::Declaration(declaration) => {
                let name = declaration_name(declaration);
                let name_ptr = env.ensure_pooled_name_ptr(name);

                if globals.contains_key(&name_ptr) {
                    return Err(PengError::new_message(
                        "duplicated global declaration".to_string(),
                    ));
                }

                let value_ptr = match declaration {
                    PengBinded::Mutable(_) => {
                        env.create_binded_stated_heap(PengBinded::Mutable(
                            PengStated::Initialized(PengValue::Nil),
                        ))
                    }

                    PengBinded::Immutable(_) => {
                        env.create_binded_stated_heap(PengBinded::Immutable(
                            PengStated::Uninitialized,
                        ))
                    }
                };

                globals.insert(name_ptr, value_ptr);
            }

            _ => {}
        }
    }

    Ok(())
}

pub fn get_allocated_global(
    env: &mut PengEnv,
    globals: &HashMap<PengNamePoolPtr, PengHeapPtr>,
    name: &str,
) -> Option<PengHeapPtr> {
    let name_ptr = env.ensure_pooled_name_ptr(name.to_string());

    if let Some(value_ptr) = globals.get(&name_ptr) {
        return Some(*value_ptr);
    }

    env.get_global_ptr_by_name_str(name)
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
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
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
                match generate_global_function(env, globals, context, function) {
                    Ok(()) => {},
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

    Ok(())
}

fn generate_global_type_value(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
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
        .push(PengInstruction::PushHeapRef(value_ptr));

    match generate_type_expression(env, globals, context, value) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::StoreHeap);
    Ok(())
}

fn generate_global_operation(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
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
        .push(PengInstruction::PushHeapRef(value_ptr));
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreHeap);

    Ok(())
}

fn generate_global_function(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
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
        .push(PengInstruction::PushHeapRef(value_ptr));
    context.push_const_and_const_instruction(value);
    context.bytecode.push(PengInstruction::StoreHeap);

    Ok(())
}