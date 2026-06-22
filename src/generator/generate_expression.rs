use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => generate_literal(env, globals, context, literal),
        PengExpression::Identifier(identifier) => generate_identifier(env, globals, context, identifier),
        PengExpression::Unary { operator, value } => {
            match generate_expression(env, globals, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
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
                Err(e) => return Err(e),
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
                        Ok(()) => {},
                        Err(e) => return Err(e),
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
                        Ok(()) => {},
                        Err(e) => return Err(e),
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
                Ok(()) => {},
                Err(e) => return Err(e),
            };
            let name = env.get_pooled_name(attribute.name.value.clone());
            context
                .bytecode
                .push(PengInstruction::GetConstAttribute(name));
            Ok(())
        }
        PengExpression::MemberAccess(member) => {
            match generate_expression(env, globals, context, &member.object) {
                Ok(()) => {},
                Err(e) => return Err(e),
            };
            let name = env.get_pooled_name(member.name.value.clone());
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
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    call: &PengMethodCallExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &call.object) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };
    let object_local = context.create_temporary_local();
    context
        .bytecode
        .push(PengInstruction::StoreLocal(object_local));

    let method = env.get_pooled_name(call.method.value.clone());
    context
        .bytecode
        .push(PengInstruction::PushLocal(object_local));
    context
        .bytecode
        .push(PengInstruction::GetConstAttribute(method));

    for generic in &call.generics {
        match generate_expression(env, globals, context, generic) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
    }

    context
        .bytecode
        .push(PengInstruction::PushLocal(object_local));
    for arg in &call.args {
        match generate_expression(env, globals, context, arg) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
    }

    context.bytecode.push(PengInstruction::FunctionCall {
        generics: call.generics.len(),
        params: call.args.len() + 1,
    });
    Ok(())
}

pub fn generate_object_construction(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    construction: &PengObjectConstructionExpression,
) -> Result<(), PengError> {
    if !construction.generics.is_empty() {
        return Err(PengError::new_positioned_message(
            "generic object construction is not supported by the current bytecode".to_string(),
            construction.object_type.position.clone(),
        ));
    }

    match generate_expression(env, globals, context, &construction.object_type) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };
    context.bytecode.push(PengInstruction::CreateObjectType);

    for field in &construction.fields {
        context.bytecode.push(PengInstruction::Duplicate);
        let name = env.get_pooled_name(field.name.value.clone());
        context
            .bytecode
            .push(PengInstruction::GetConstAttributeRef(name));
        match generate_expression(env, globals, context, &field.value) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
        context.bytecode.push(PengInstruction::StoreValue);
    }

    Ok(())
}

pub fn generate_try_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    value: &PengPositionedExpression,
    elsing: Option<&PengPositionedExpression>,
) -> Result<(), PengError> {
    let call = match &value.value {
        PengExpression::FuncCall(call) => call,
        _ => {
            return Err(PengError::new_positioned_message(
                "'try' currently requires a function call".to_string(),
                value.position.clone(),
            ));
        }
    };

    match generate_expression(env, globals, context, &call.function) {
        Ok(()) => {},
        Err(e) => return Err(e),
    };
    for generic in &call.generics {
        match generate_expression(env, globals, context, generic) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
    }
    for arg in &call.args {
        match generate_expression(env, globals, context, arg) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
    }
    context.bytecode.push(PengInstruction::TryFunctionCall {
        generics: call.generics.len(),
        params: call.args.len(),
    });

    let success_jump = context.bytecode.len();
    context
        .bytecode
        .push(PengInstruction::JumpIfTrue(usize::MAX));
    context.bytecode.push(PengInstruction::Pop);

    match elsing {
        Some(PengPositioned {
            value: PengExpression::Try { value, .. },
            ..
        }) => match generate_expression(env, globals, context, value) {
            Ok(()) => {},
            Err(e) => return Err(e),
        },
        Some(elsing) => match generate_expression(env, globals, context, elsing) {
            Ok(()) => {},
            Err(e) => return Err(e),
        },
        None => context.push_const_and_const_instruction(PengValue::Nil),
    }

    let end = context.bytecode.len();
    context.bytecode[success_jump] = PengInstruction::JumpIfTrue(end);
    Ok(())
}

pub fn generate_index_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    index: &PengIndexExpression,
) -> Result<(), PengError> {
    match generate_expression(env, globals, context, &index.object) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, globals, context, &index.index) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::GetIndex);
    Ok(())
}

pub fn generate_type_expression(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    type_expression: &PengPositionedTypeExpression,
) -> Result<(), PengError> {
    match &type_expression.value {
        PengTypeExpression::Union(types) => {
            for typ in types {
                match generate_type_expression(env, globals, context, typ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            context
                .bytecode
                .push(PengInstruction::CreateUnion(types.len()));
            Ok(())
        }
        PengTypeExpression::Custom(expression) => generate_expression(env, globals, context, expression),
        PengTypeExpression::Vector(inner) => {
            let inner_type = match static_type_from_expression(inner) {
                Some(inner_type) => inner_type,
                None => {
                    return Err(PengError::new_positioned_message(
                        "vector type requires a static inner type".to_string(),
                        inner.position.clone(),
                    ));
                }
            };

            context.push_const_and_const_instruction(PengValue::Type(PengType::Vector(Box::new(
                inner_type,
            ))));
            Ok(())
        }
        PengTypeExpression::TypeLiteral(literal) => generate_type_literal(env, globals, context, literal),
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

            context.push_const_and_const_instruction(PengValue::Type(typ));
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
