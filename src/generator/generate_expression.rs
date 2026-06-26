use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => generate_literal(env, globals, context, literal),
        PengExpression::Identifier(identifier) => {
            generate_identifier(env, globals, context, identifier)
        }
        PengExpression::Unary { operator, value } => {
            match generate_expression(env, globals, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            }

            match operator {
                PengUnaryOperator::Negate => {
                    context.bytecode.push(PengInstruction::Negate);
                }
                PengUnaryOperator::Not => {
                    context.bytecode.push(PengInstruction::Not);
                }
            }

            Ok(())
        }
        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            match generate_expression(env, globals, context, left) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            }

            match operator {
                PengBinaryOperator::ShortCircuitAnd | PengBinaryOperator::ShortCircuitOr => {
                    context.bytecode.push(PengInstruction::Duplicate);
                    let jump_index = context.bytecode.len();
                    context.bytecode.push(match operator {
                        PengBinaryOperator::ShortCircuitAnd => {
                            PengInstruction::JumpIfFalse(usize::MAX)
                        }
                        PengBinaryOperator::ShortCircuitOr => {
                            PengInstruction::JumpIfTrue(usize::MAX)
                        }
                        _ => unreachable!(),
                    });
                    context.bytecode.push(PengInstruction::Pop);

                    match generate_expression(env, globals, context, right) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_expression".to_string(),
                            )));
                        }
                    };

                    let target = context.bytecode.len();
                    context.bytecode[jump_index] = match operator {
                        PengBinaryOperator::ShortCircuitAnd => PengInstruction::JumpIfFalse(target),
                        PengBinaryOperator::ShortCircuitOr => PengInstruction::JumpIfTrue(target),
                        _ => unreachable!(),
                    };
                }
                _ => {
                    match generate_expression(env, globals, context, right) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generate_expression".to_string(),
                            )));
                        }
                    };
                    generate_binary_operator(context, operator);
                }
            }

            Ok(())
        }
        PengExpression::Type(type_expression) => {
            generate_type_expression(env, globals, context, type_expression)
        }
        PengExpression::FuncCall(call) => generate_function_call(env, globals, context, call),
        PengExpression::MethodCall(call) => generate_method_call(env, globals, context, call),
        PengExpression::AttributeAccess(attribute) => {
            match generate_expression(env, globals, context, &attribute.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            };
            let name = env.ensure_pooled_name_ptr(attribute.name.value.clone());
            context
                .bytecode
                .push(PengInstruction::GetConstAttribute(name));
            Ok(())
        }
        PengExpression::MemberAccess(member) => {
            match generate_expression(env, globals, context, &member.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generate_expression".to_string(),
                    )));
                }
            };
            let name = env.ensure_pooled_name_ptr(member.name.value.clone());
            context.bytecode.push(PengInstruction::GetConstMember(name));
            Ok(())
        }
        PengExpression::Index(index) => generate_index_expression(env, globals, context, index),
        PengExpression::ObjectConstruction(construction) => {
            generate_object_construction(env, globals, context, construction)
        }
        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => generate_operation_call(env, globals, context, left, operation, right),
        PengExpression::Try { value, elsing } => {
            generate_try_expression(env, globals, context, value, elsing.as_deref())
        }
    }
}

pub fn generate_method_call(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    call: &PengMethodCallExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &call.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    };
    let object_local = context.create_temporary_local();
    context
        .bytecode
        .push(PengInstruction::StoreLocal(object_local));

    let method = env.ensure_pooled_name_ptr(call.method.value.clone());
    context
        .bytecode
        .push(PengInstruction::PushLocal(object_local));
    context
        .bytecode
        .push(PengInstruction::GetConstAttribute(method));

    context
        .bytecode
        .push(PengInstruction::PushLocal(object_local));

    for arg in &call.args {
        match generate_expression(env, globals, context, &arg.value.expression) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_expression".to_string(),
                )));
            }
        }
    }

    context
        .bytecode
        .push(PengInstruction::FunctionCall(call.args.len() + 1));

    Ok(())
}

pub fn generate_object_construction(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    construction: &PengObjectConstructionExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &construction.object_type) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    context.bytecode.push(PengInstruction::CreateTypedObject);

    for field in &construction.fields {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(field.name.value.clone());

        match generate_expression(env, globals, context, &field.value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generate_expression".to_string(),
                )));
            }
        }

        context
            .bytecode
            .push(PengInstruction::SetConstAttribute(name));
    }

    Ok(())
}

pub fn generate_try_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    value: &PengPositionedExpression,
    elsing: Option<&PengPositionedExpression>,
) -> Result<(), PengError> {
    match &value.value {
        PengExpression::FuncCall(call) => {
            match generate_expression(env, globals, context, &call.function) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try function call".to_string(),
                    )));
                }
            }

            let variadic_index =
                match generate_function_call_args(env, globals, context, &call.args) {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating try function call args".to_string(),
                        )));
                    }
                };

            match variadic_index {
                Some(index) => {
                    context
                        .bytecode
                        .push(PengInstruction::TryFunctionCallSpread(index));
                }

                None => {
                    context
                        .bytecode
                        .push(PengInstruction::TryFunctionCall(call.args.len()));
                }
            }
        }

        PengExpression::MethodCall(call) => {
            match generate_expression(env, globals, context, &call.object) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try method call object".to_string(),
                    )));
                }
            }

            let object_local = context.create_temporary_local();

            context
                .bytecode
                .push(PengInstruction::StoreLocal(object_local));

            let method = env.ensure_pooled_name_ptr(call.method.value.clone());

            context
                .bytecode
                .push(PengInstruction::PushLocal(object_local));

            context
                .bytecode
                .push(PengInstruction::GetConstAttribute(method));

            context
                .bytecode
                .push(PengInstruction::PushLocal(object_local));

            let variadic_index =
                match generate_function_call_args(env, globals, context, &call.args) {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating try method call args".to_string(),
                        )));
                    }
                };

            match variadic_index {
                Some(index) => {
                    context
                        .bytecode
                        .push(PengInstruction::TryFunctionCallSpread(index + 1));
                }

                None => {
                    context
                        .bytecode
                        .push(PengInstruction::TryFunctionCall(call.args.len() + 1));
                }
            }
        }

        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => {
            match generate_expression(env, globals, context, operation) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call operation".to_string(),
                    )));
                }
            }

            match generate_expression(env, globals, context, left) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call left".to_string(),
                    )));
                }
            }

            match generate_expression(env, globals, context, right) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating try operation call right".to_string(),
                    )));
                }
            }

            context.bytecode.push(PengInstruction::TryOperationCall);
        }

        _ => {
            return Err(PengError::new_positioned_message(
                "expected function, method or operation call after try".to_string(),
                value.position.clone(),
            ));
        }
    }

    let success_jump = context.bytecode.len();

    context
        .bytecode
        .push(PengInstruction::JumpIfTrue(usize::MAX));

    context.bytecode.push(PengInstruction::Pop);

    match elsing {
        Some(elsing) => match generate_expression(env, globals, context, elsing) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating try else expression".to_string(),
                )));
            }
        },

        None => {
            context.push_const_and_const_instruction(PengValue::Cell(PengCell::Nil));
        }
    }

    let end = context.bytecode.len();
    context.bytecode[success_jump] = PengInstruction::JumpIfTrue(end);

    Ok(())
}

pub fn generate_index_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    index: &PengIndexExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &index.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    match generate_expression(env, globals, context, &index.index) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_expression".to_string(),
            )));
        }
    }

    context.bytecode.push(PengInstruction::GetIndex);
    Ok(())
}

pub fn generate_type_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    type_expression: &PengPositionedTypeExpression,
) -> Result<(), PengError> {
    match &type_expression.value {
        PengTypeExpression::Union(types) => {
            for typ in types {
                match generate_type_expression(env, globals, context, typ) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generate_expression".to_string(),
                        )));
                    }
                }
            }

            context
                .bytecode
                .push(PengInstruction::CreateUnion(types.len()));

            Ok(())
        }

        PengTypeExpression::Custom(expression) => {
            generate_expression(env, globals, context, expression)
        }

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

            context.push_const_and_const_instruction(PengValue::Heap(PengHeapValue::Type(
                PengType::Vector(Box::new(inner_type)),
            )));

            Ok(())
        }

        PengTypeExpression::TypeLiteral(literal) => {
            generate_type_literal(env, globals, context, literal)
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

            context.push_const_and_const_instruction(PengValue::Heap(PengHeapValue::Type(typ)));

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
