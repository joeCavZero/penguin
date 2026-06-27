use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengPositionedLiteral,
) -> Result<(), PengError> {
    match &literal.value {
        PengLiteral::Type(type_literal) => {
            return generate_type_literal(env, context, type_literal, literal.position.clone());
        }
        PengLiteral::Module(module) => {
            return generate_module_literal(env, context, module, literal.position.clone());
        }
        PengLiteral::Vector(values) => {
            return generate_vector_literal(env, context, values, literal.position.clone());
        }
        PengLiteral::Object(fields) => {
            return generate_object_literal(env, context, fields, literal.position.clone());
        }
        _ => {}
    }

    let value = match &literal.value {
        PengLiteral::Nil => PengValue::Cell(PengCell::Nil),
        PengLiteral::Int(value) => PengValue::Cell(PengCell::Int(*value)),
        PengLiteral::Uint(value) => PengValue::Cell(PengCell::Uint(*value)),
        PengLiteral::Byte(value) => PengValue::Cell(PengCell::Byte(*value)),
        PengLiteral::Float32(value) => PengValue::Cell(PengCell::Float32(*value)),
        PengLiteral::Float64(value) => PengValue::Cell(PengCell::Float64(*value)),
        PengLiteral::Bool(value) => PengValue::Cell(PengCell::Bool(*value)),
        PengLiteral::String(value) => PengValue::Box(PengBox::String(value.clone())),
        PengLiteral::Type(_) => unreachable!(),
        PengLiteral::Function(function) => {
            match generate_function_value(
                env,
                context,
                &function.params,
                &function.body,
                literal.position.clone(),
            ) {
                Ok(value) => PengValue::Box(value),
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_literal".to_string(),
                    )));
                }
            }
        }
        PengLiteral::Module(_) => unreachable!(),
        PengLiteral::Vector(_) => unreachable!(),
        PengLiteral::Operation(operation) => {
            match generate_operation_value(
                env,
                context,
                &operation.params,
                &operation.body,
                literal.position.clone(),
            ) {
                Ok(value) => PengValue::Box(value),
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_literal".to_string(),
                    )));
                }
            }
        }
        PengLiteral::Object(_) => unreachable!(),
    };

    context.push_const_and_const_instruction(env, value, literal.position.clone());
    Ok(())
}

pub fn generate_type_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
    pos: PengPosition,
) -> Result<(), PengError> {
    match generate_type_literal_after_base(env, context, literal, pos) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_object_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    fields: &[PengObjectFieldLiteral],
    pos: PengPosition,
) -> Result<(), PengError> {
    context.push_positioned_instruction(PengInstruction::CreateEmptyObject, pos);

    for field in fields {
        context
            .push_positioned_instruction(PengInstruction::Duplicate, field.name.position.clone());

        let name = env.ensure_pooled_name_ptr(field.name.value.clone());

        match generate_expression(env, context, &field.value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        };
        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            field.name.position.clone(),
        );
    }

    Ok(())
}

pub fn generate_type_literal_after_base(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
    pos: PengPosition,
) -> Result<(), PengError> {
    for super_type in &literal.supers {
        match generate_expression(env, context, super_type) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        }
    }

    context
        .push_positioned_instruction(PengInstruction::CreateSuperType(literal.supers.len()), pos);

    for field in &literal.fields {
        context.push_positioned_instruction(PengInstruction::Duplicate, field.position.clone());

        let name = env.ensure_pooled_name_ptr(field.value.name.value.clone());

        match &field.value.value {
            Some(value) => match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_literal".to_string(),
                    )));
                }
            },
            None => context.push_const_and_const_instruction(
                env,
                PengValue::Cell(PengCell::Nil),
                field.position.clone(),
            ),
        }

        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            field.position.clone(),
        );
    }

    for function in &literal.functions {
        context.push_positioned_instruction(PengInstruction::Duplicate, function.position.clone());

        let name = env.ensure_pooled_name_ptr(function.value.name.value.clone());

        let value = match generate_function_declaration_value(env, context, function) {
            Ok(value) => value,
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        };

        context.push_const_and_const_instruction(
            env,
            PengValue::Box(value),
            function.position.clone(),
        );

        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            function.position.clone(),
        );
    }

    Ok(())
}

pub fn generate_vector_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    values: &[PengPositionedExpression],
    pos: PengPosition,
) -> Result<(), PengError> {
    for value in values {
        match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        };
    }

    context.push_positioned_instruction(PengInstruction::CreateVector(values.len()), pos);

    Ok(())
}

pub fn generate_module_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    module: &PengModuleLiteral,
    pos: PengPosition,
) -> Result<(), PengError> {
    context.push_positioned_instruction(PengInstruction::CreateEmptyModule, pos);

    for binded_declaration in &module.body {
        let declaration = match binded_declaration {
            PengBinded::Mutable(declaration) | PengBinded::Immutable(declaration) => declaration,
        };

        let name = match declaration {
            PengDeclaration::Var(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
        };

        let name_ptr = env.ensure_pooled_name_ptr(name);

        let member_pos = declaration_position(declaration);
        context.push_positioned_instruction(PengInstruction::Duplicate, member_pos.clone());

        match declaration {
            PengDeclaration::Var(declaration) => match &declaration.value.value {
                Some(value) => match generate_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                },
                None => context.push_const_and_const_instruction(
                    env,
                    PengValue::Cell(PengCell::Nil),
                    member_pos.clone(),
                ),
            },

            PengDeclaration::As(declaration) => {
                match generate_expression(env, context, &declaration.value.value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                };
            }

            PengDeclaration::Function(declaration) => {
                let value = match generate_function_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }

            PengDeclaration::Type(declaration) => match &declaration.value.value {
                Some(value) => match generate_type_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                },
                None => {
                    let literal = PengTypeLiteral {
                        supers: declaration.value.supers.clone(),
                        fields: declaration.value.fields.clone(),
                        functions: declaration.value.functions.clone(),
                    };

                    match generate_type_literal(env, context, &literal, member_pos.clone()) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_literal".to_string(),
                            )));
                        }
                    };
                }
            },

            PengDeclaration::Module(declaration) => {
                match generate_module_declaration_value(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                };
            }

            PengDeclaration::Operation(declaration) => {
                let value = match generate_operation_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_literal".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }
        }

        context.push_positioned_instruction(PengInstruction::SetMember(name_ptr), member_pos);
    }

    Ok(())
}
