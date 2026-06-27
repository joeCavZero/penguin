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
            return generate_type_literal(env, context, type_literal);
        }
        PengLiteral::Module(module) => {
            return generate_module_literal(env, context, module);
        }
        PengLiteral::Vector(values) => {
            return generate_vector_literal(env, context, values);
        }
        PengLiteral::Object(fields) => {
            return generate_object_literal(env, context, fields);
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
            match generate_function_value(env, context, &function.params, &function.body) {
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
            match generate_operation_value(env, context, &operation.params, &operation.body) {
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

    context.push_const_and_const_instruction(env, value);
    Ok(())
}

pub fn generate_type_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
) -> Result<(), PengError> {
    match generate_type_literal_after_base(env, context, literal) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_object_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    fields: &[PengObjectFieldLiteral],
) -> Result<(), PengError> {
    context.bytecode.push(PengInstruction::CreateEmptyObject);

    for field in fields {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(field.name.value.clone());

        match generate_expression(env, context, &field.value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        };
        context
            .bytecode
            .push(PengInstruction::SetAttribute(name));
    }

    Ok(())
}

pub fn generate_type_literal_after_base(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
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
        .bytecode
        .push(PengInstruction::CreateSuperType(literal.supers.len()));

    for field in &literal.fields {
        context.bytecode.push(PengInstruction::Duplicate);

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
            None => context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil)),
        }

        context
            .bytecode
            .push(PengInstruction::SetAttribute(name));
    }

    for function in &literal.functions {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(function.value.name.value.clone());

        let value = match generate_function_declaration_value(env, context, function) {
            Ok(value) => value,
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_literal".to_string(),
                )));
            }
        };

        context.push_const_and_const_instruction(env, PengValue::Box(value));

        context
            .bytecode
            .push(PengInstruction::SetAttribute(name));
    }

    Ok(())
}

pub fn generate_vector_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    values: &[PengPositionedExpression],
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

    context
        .bytecode
        .push(PengInstruction::CreateVector(values.len()));

    Ok(())
}

pub fn generate_module_literal(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    module: &PengModuleLiteral,
) -> Result<(), PengError> {
    context.bytecode.push(PengInstruction::CreateEmptyModule);

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

        context.bytecode.push(PengInstruction::Duplicate);

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
                None => context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil)),
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
                context.push_const_and_const_instruction(env, PengValue::Box(value));
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

                    match generate_type_literal(env, context, &literal) {
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
                context.push_const_and_const_instruction(env, PengValue::Box(value));
            }
        }

        context
            .bytecode
            .push(PengInstruction::SetMember(name_ptr));
    }

    Ok(())
}
