use crate::core::*;
pub use crate::generator::generate::*;

use crate::parser::*;

pub fn generate_program(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
) -> Result<PengUnit, PengError> {
    let using_unit = PengUnit::library();

    generate_program_using(env, declarations, &using_unit)
}

pub fn generate_program_using(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
    using_unit: &PengUnit,
) -> Result<PengUnit, PengError> {
    let mut context = PengGeneratorContext::new();
    context.use_globals(env, using_unit.globals().clone());

    match allocate_program_globals(env, declarations, &mut context) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    }

    match generate_program_initialization(env, declarations, &mut context) {
        Ok(()) => {}
        Err(e) => return Err(e),
    };

    let globals = context.globals().clone();

    let terminal_pos = match declarations.last() {
        Some(declaration) => match declaration {
            PengBinded::Mutable(declaration) | PengBinded::Immutable(declaration) => {
                declaration_position(declaration)
            }
        },
        // An empty program has no source node to associate with its synthetic return.
        None => PengPosition::new(0, 0, None),
    };
    let program_init = create_anonymous_bytecode_function(
        env,
        context,
        PengBytecodeFunctionParams::Fixed(0),
        terminal_pos,
    );

    Ok(PengUnit::new(
        program_init,
        globals,
        using_unit.custom_access().clone(),
        using_unit.custom_add().cloned(),
        using_unit.custom_subtract().cloned(),
        using_unit.custom_multiply().cloned(),
        using_unit.custom_divide().cloned(),
        using_unit.custom_power().cloned(),
        using_unit.custom_remainder().cloned(),
        using_unit.custom_negate().cloned(),
        using_unit.custom_concat().cloned(),
        using_unit.custom_and().cloned(),
        using_unit.custom_or().cloned(),
        using_unit.custom_not().cloned(),
        using_unit.custom_equals().cloned(),
        using_unit.custom_not_equals().cloned(),
        using_unit.custom_greater_than().cloned(),
        using_unit.custom_greater_equals_than().cloned(),
        using_unit.custom_less_than().cloned(),
        using_unit.custom_less_equals_than().cloned(),
    ))
}

fn allocate_program_globals(
    env: &mut PengEnv,
    declarations: &Vec<PengBindedDeclaration>,
    context: &mut PengGeneratorContext,
) -> Result<(), PengError> {
    for declaration in declarations {
        let name = declaration_name(declaration);
        let name_ptr = env.ensure_pooled_name_ptr(name);

        let heap_ptr = env.create_heap_value(PengValue::Cell(PengCell::Nil));

        let value = match declaration {
            PengBinded::Mutable(_) => PengBinded::Mutable(heap_ptr),

            PengBinded::Immutable(_) => PengBinded::Immutable(heap_ptr),
        };

        context.insert_global(name_ptr, value);
    }

    Ok(())
}

pub fn get_allocated_global(
    env: &mut PengEnv,
    context: &PengGeneratorContext,
    name: &str,
) -> Option<PengBindedHeapPtr> {
    let name_ptr = env.ensure_pooled_name_ptr(name.to_string());

    match context.get_global(name_ptr) {
        Some(value) => Some(value.clone()),
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
            PengDeclaration::Var(variable) => {
                match generate_global_variable(env, context, variable) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating global variable".to_string(),
                        )));
                    }
                }
            }
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
                match generate_global_type(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_program".to_string(),
                        )));
                    }
                }
            }
            PengDeclaration::Module(module) => match generate_global_module(env, context, module) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_program".to_string(),
                    )));
                }
            },
            PengDeclaration::As(declaration) => {
                match generate_global_as(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating global as".to_string(),
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

fn generate_global_variable(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedVariableDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global variable was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(value_ptr),
        declaration.position.clone(),
    );

    match &declaration.value.value {
        Some(value) => match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => return Err(e),
        },

        None => {
            context.push_const_and_const_instruction(
                env,
                PengValue::Cell(PengCell::Nil),
                declaration.position.clone(),
            );
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    Ok(())
}

fn generate_global_operation(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedOperationDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global operation was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_operation_declaration_value(env, context, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    };

    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(value_ptr),
        declaration.position.clone(),
    );
    context.push_const_and_const_instruction(
        env,
        PengValue::Box(value),
        declaration.position.clone(),
    );
    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    Ok(())
}

fn generate_global_function(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global function was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    let value = match generate_function_declaration_value(env, context, declaration) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_program".to_string(),
            )));
        }
    };

    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(value_ptr),
        declaration.position.clone(),
    );
    context.push_const_and_const_instruction(
        env,
        PengValue::Box(value),
        declaration.position.clone(),
    );
    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    Ok(())
}

fn generate_global_module(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedModuleDeclaration,
) -> Result<(), PengError> {
    let module_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global module was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(module_ptr),
        declaration.position.clone(),
    );
    context.push_positioned_instruction(
        PengInstruction::CreateEmptyModule,
        declaration.position.clone(),
    );
    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    match generate_module_members_into_existing_module(
        env,
        context,
        module_ptr,
        &declaration.value.body,
    ) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating global module members".to_string(),
            )));
        }
    }

    Ok(())
}

fn generate_module_members_into_existing_module(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    module_ptr: PengHeapPtr,
    body: &Vec<PengBindedDeclaration>,
) -> Result<(), PengError> {
    context.push_scope();

    let result = generate_module_members_into_existing_module_scope(env, context, module_ptr, body);

    context.pop_scope();

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

fn generate_module_members_into_existing_module_scope(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    module_ptr: PengHeapPtr,
    body: &Vec<PengBindedDeclaration>,
) -> Result<(), PengError> {
    for binded_declaration in body {
        let (declaration, immutable) = match binded_declaration {
            PengBinded::Mutable(declaration) => (declaration, false),
            PengBinded::Immutable(declaration) => (declaration, true),
        };
        let pos = declaration_position(declaration);
        let local = generate_reserved_temporary_local(env, context, pos.clone());

        let name = declaration_name(binded_declaration);
        let name_ptr = env.ensure_pooled_name_ptr(name.clone());

        context.push_positioned_instruction(PengInstruction::PushHeapRef(module_ptr), pos.clone());

        match declaration {
            PengDeclaration::Var(declaration) => match &declaration.value.value {
                Some(value) => match generate_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                None => {
                    context.push_const_and_const_instruction(
                        env,
                        PengValue::Cell(PengCell::Nil),
                        pos.clone(),
                    );
                }
            },

            PengDeclaration::As(declaration) => {
                match generate_expression(env, context, &declaration.value.value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            PengDeclaration::Function(declaration) => {
                let value = match generate_function_declaration_value(env, context, declaration) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                context.push_const_and_const_instruction(env, PengValue::Box(value), pos.clone());
            }

            PengDeclaration::Operation(declaration) => {
                let value = match generate_operation_declaration_value(env, context, declaration) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                context.push_const_and_const_instruction(env, PengValue::Box(value), pos.clone());
            }

            PengDeclaration::Type(declaration) => match &declaration.value.value {
                Some(value) => match generate_type_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                None => {
                    let literal = PengTypeLiteral {
                        supers: declaration.value.supers.clone(),
                        fields: declaration.value.fields.clone(),
                        functions: declaration.value.functions.clone(),
                    };

                    match generate_type_literal(env, context, &literal, pos.clone()) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }
                }
            },

            PengDeclaration::Module(declaration) => {
                match generate_module_declaration_value(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        generate_make_immutable_if_needed(context, immutable, pos.clone());
        context.push_positioned_instruction(PengInstruction::Duplicate, pos.clone());
        context.push_positioned_instruction(PengInstruction::StoreLocal(local), pos.clone());
        context.push_positioned_instruction(PengInstruction::SetMember(name_ptr), pos);
        context.insert_local(name, local);
    }

    Ok(())
}

fn generate_global_type(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global type was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    match &declaration.value.value {
        Some(value) => {
            context.push_positioned_instruction(
                PengInstruction::PushHeapRef(value_ptr),
                declaration.position.clone(),
            );

            match generate_type_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating global type".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(
                PengInstruction::StoreHeap,
                declaration.position.clone(),
            );
        }

        None => match generate_structured_global_type(env, context, value_ptr, declaration) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating structured global type".to_string(),
                )));
            }
        },
    }

    Ok(())
}

fn generate_global_as(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
) -> Result<(), PengError> {
    let value_ptr = match get_allocated_global(env, context, &declaration.value.name.value) {
        Some(value) => *value.value(),
        None => {
            return Err(PengError::new_positioned_message(
                "global as was not allocated".to_string(),
                declaration.value.name.position.clone(),
            ));
        }
    };

    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(value_ptr),
        declaration.position.clone(),
    );

    match generate_expression(env, context, &declaration.value.value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating global as".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    Ok(())
}
