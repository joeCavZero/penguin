use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_expression(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => generate_literal(env, context, literal),
        PengExpression::Identifier(identifier) => generate_identifier(env, context, identifier),
        PengExpression::Unary { operator, value } => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            }

            match operator {
                PengUnaryOperator::Negate => {
                    context.push_positioned_instruction(
                        PengInstruction::Negate,
                        expression.position.clone(),
                    );
                }
                PengUnaryOperator::Not => {
                    context.push_positioned_instruction(
                        PengInstruction::Not,
                        expression.position.clone(),
                    );
                }
            }

            Ok(())
        }
        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            match generate_expression(env, context, left) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            }

            match operator {
                PengBinaryOperator::ShortCircuitAnd | PengBinaryOperator::ShortCircuitOr => {
                    context.push_positioned_instruction(
                        PengInstruction::Duplicate,
                        expression.position.clone(),
                    );
                    let jump_index = context.bytecode.len();
                    context.push_positioned_instruction(
                        match operator {
                            PengBinaryOperator::ShortCircuitAnd => {
                                PengInstruction::JumpIfFalse(usize::MAX)
                            }
                            PengBinaryOperator::ShortCircuitOr => {
                                PengInstruction::JumpIfTrue(usize::MAX)
                            }
                            _ => unreachable!(),
                        },
                        expression.position.clone(),
                    );
                    context.push_positioned_instruction(
                        PengInstruction::Pop,
                        expression.position.clone(),
                    );

                    match generate_expression(env, context, right) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_expression".to_string(),
                            )));
                        }
                    };

                    let target = context.bytecode.len();
                    context.patch_jump(jump_index, target);
                }
                _ => {
                    match generate_expression(env, context, right) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_expression".to_string(),
                            )));
                        }
                    };
                    generate_binary_operator(context, operator, expression.position.clone());
                }
            }

            Ok(())
        }
        PengExpression::Type(type_expression) => {
            generate_type_expression(env, context, type_expression)
        }
        PengExpression::FuncCall(call) => {
            generate_function_call(env, context, call, expression.position.clone())
        }
        PengExpression::MethodCall(call) => {
            generate_method_call(env, context, call, expression.position.clone())
        }
        PengExpression::AttributeAccess(attribute) => {
            match generate_expression(env, context, &attribute.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            };
            let name = env.ensure_pooled_name_ptr(attribute.name.value.clone());
            context.push_positioned_instruction(
                PengInstruction::GetAttribute(name),
                expression.position.clone(),
            );
            Ok(())
        }
        PengExpression::MemberAccess(member) => {
            match generate_expression(env, context, &member.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            };
            let name = env.ensure_pooled_name_ptr(member.name.value.clone());
            context.push_positioned_instruction(
                PengInstruction::GetMember(name),
                expression.position.clone(),
            );
            Ok(())
        }
        PengExpression::Index(index) => {
            generate_index_expression(env, context, index, expression.position.clone())
        }
        PengExpression::ObjectConstruction(construction) => {
            generate_object_construction(env, context, construction, expression.position.clone())
        }
        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => generate_operation_call(
            env,
            context,
            left,
            operation,
            right,
            expression.position.clone(),
        ),
        PengExpression::Try { value, elsing } => generate_try_expression(
            env,
            context,
            value,
            elsing.as_deref(),
            expression.position.clone(),
        ),
    }
}

pub fn generate_method_call(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    call: &PengMethodCallExpression,
    pos: PengPosition,
) -> Result<(), PengError> {
    let object_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &call.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    };

    context.push_positioned_instruction(PengInstruction::StoreLocal(object_local), pos.clone());

    let method = env.ensure_pooled_name_ptr(call.method.value.clone());

    context.push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
    context.push_positioned_instruction(PengInstruction::GetAttribute(method), pos.clone());
    context.push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());

    for arg in &call.args {
        match generate_expression(env, context, &arg.value.expression) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_expression".to_string(),
                )));
            }
        }
    }

    context.push_positioned_instruction(PengInstruction::FunctionCall(call.args.len() + 1), pos);

    Ok(())
}

pub fn generate_object_construction(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    construction: &PengObjectConstructionExpression,
    pos: PengPosition,
) -> Result<(), PengError> {
    match generate_expression(env, context, &construction.object_type) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::CreateTypedObject, pos.clone());

    for field in &construction.fields {
        context
            .push_positioned_instruction(PengInstruction::Duplicate, field.name.position.clone());

        let name = env.ensure_pooled_name_ptr(field.name.value.clone());

        match generate_expression(env, context, &field.value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_expression".to_string(),
                )));
            }
        }

        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            field.name.position.clone(),
        );
    }

    Ok(())
}

pub fn generate_try_expression(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    value: &PengPositionedExpression,
    elsing: Option<&PengPositionedExpression>,
    pos: PengPosition,
) -> Result<(), PengError> {
    match &value.value {
        PengExpression::FuncCall(call) => {
            match generate_expression(env, context, &call.function) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try function call".to_string(),
                    )));
                }
            }

            let variadic_index = match generate_function_call_args(env, context, &call.args) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try function call args".to_string(),
                    )));
                }
            };

            match variadic_index {
                Some(index) => {
                    context.push_positioned_instruction(
                        PengInstruction::TryFunctionCallSpread(index),
                        pos.clone(),
                    );
                }

                None => {
                    context.push_positioned_instruction(
                        PengInstruction::TryFunctionCall(call.args.len()),
                        pos.clone(),
                    );
                }
            }
        }

        PengExpression::MethodCall(call) => {
            let object_local = generate_reserved_temporary_local(env, context, pos.clone());

            match generate_expression(env, context, &call.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try method call object".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(
                PengInstruction::StoreLocal(object_local),
                pos.clone(),
            );

            let method = env.ensure_pooled_name_ptr(call.method.value.clone());

            context
                .push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
            context.push_positioned_instruction(PengInstruction::GetAttribute(method), pos.clone());
            context
                .push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());

            let variadic_index = match generate_function_call_args(env, context, &call.args) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try method call args".to_string(),
                    )));
                }
            };

            match variadic_index {
                Some(index) => {
                    context.push_positioned_instruction(
                        PengInstruction::TryFunctionCallSpread(index + 1),
                        pos.clone(),
                    );
                }

                None => {
                    context.push_positioned_instruction(
                        PengInstruction::TryFunctionCall(call.args.len() + 1),
                        pos.clone(),
                    );
                }
            }
        }

        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => {
            match generate_expression(env, context, operation) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call operation".to_string(),
                    )));
                }
            }

            match generate_expression(env, context, left) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call left".to_string(),
                    )));
                }
            }

            match generate_expression(env, context, right) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call right".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(PengInstruction::TryOperationCall, pos.clone());
        }

        _ => {
            return Err(PengError::new_positioned_message(
                "expected function, method or operation call after try".to_string(),
                value.position.clone(),
            ));
        }
    }

    let success_jump = context.bytecode.len();

    context.push_positioned_instruction(PengInstruction::JumpIfTrue(usize::MAX), pos.clone());
    context.push_positioned_instruction(PengInstruction::Pop, pos.clone());

    match elsing {
        Some(elsing) => match generate_expression(env, context, elsing) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating try else expression".to_string(),
                )));
            }
        },

        None => {
            context.push_const_and_const_instruction(env, PengValue::Cell(PengCell::Nil), pos);
        }
    }

    let end = context.bytecode.len();
    context.patch_jump(success_jump, end);

    Ok(())
}

pub fn generate_index_expression(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    index: &PengIndexExpression,
    pos: PengPosition,
) -> Result<(), PengError> {
    match generate_expression(env, context, &index.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    match generate_expression(env, context, &index.index) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::GetIndex, pos);
    Ok(())
}

pub fn generate_type_expression(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    type_expression: &PengPositionedTypeExpression,
) -> Result<(), PengError> {
    match &type_expression.value {
        PengTypeExpression::Union(types) => {
            for typ in types {
                match generate_type_expression(env, context, typ) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_expression".to_string(),
                        )));
                    }
                }
            }

            context.push_positioned_instruction(
                PengInstruction::CreateUnion(types.len()),
                type_expression.position.clone(),
            );

            Ok(())
        }

        PengTypeExpression::Custom(expression) => generate_expression(env, context, expression),

        PengTypeExpression::Vector(inner) => {
            let inner_type = match static_type_from_expression(inner) {
                Some(inner_type) => inner_type,
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected static vector inner type".to_string(),
                        inner.position.clone(),
                    ));
                }
            };

            context.push_const_and_const_instruction(
                env,
                PengValue::Box(PengBox::Type(PengType::Vector(Box::new(inner_type)))),
                type_expression.position.clone(),
            );

            Ok(())
        }

        PengTypeExpression::TypeLiteral(literal) => {
            generate_type_literal(env, context, literal, type_expression.position.clone())
        }

        _ => {
            let typ = match static_type_from_expression(type_expression) {
                Some(typ) => typ,
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected type value".to_string(),
                        type_expression.position.clone(),
                    ));
                }
            };

            context.push_const_and_const_instruction(
                env,
                PengValue::Box(PengBox::Type(typ)),
                type_expression.position.clone(),
            );

            Ok(())
        }
    }
}

pub fn static_type_from_expression(
    type_expression: &PengPositionedTypeExpression,
) -> Option<PengType> {
    match &type_expression.value {
        PengTypeExpression::Nil => Some(PengType::Nil),
        PengTypeExpression::Int => Some(PengType::Int),
        PengTypeExpression::Uint => Some(PengType::Uint),
        PengTypeExpression::Float32 => Some(PengType::Float32),
        PengTypeExpression::Float64 => Some(PengType::Float64),
        PengTypeExpression::Byte => Some(PengType::Byte),
        PengTypeExpression::Bool => Some(PengType::Bool),
        PengTypeExpression::String => Some(PengType::String),
        PengTypeExpression::Object => Some(PengType::Object),
        PengTypeExpression::Type => Some(PengType::Type),
        PengTypeExpression::Module => Some(PengType::Module),
        PengTypeExpression::Function => Some(PengType::Function),
        PengTypeExpression::Operation => Some(PengType::Operator),
        PengTypeExpression::Thread => Some(PengType::Thread),
        PengTypeExpression::Any => Some(PengType::Any),
        PengTypeExpression::UnionType => Some(PengType::Union),
        PengTypeExpression::Vector(inner) => match static_type_from_expression(inner) {
            Some(inner_type) => Some(PengType::Vector(Box::new(inner_type))),
            None => None,
        },
        PengTypeExpression::TypeLiteral(_) => Some(PengType::Type),
        PengTypeExpression::Custom(_) => None,
        PengTypeExpression::Union(_) => None,
    }
}
