use crate::core::*;
use crate::parser::*;
use crate::generator::*;


pub fn generate_expression(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => {
            generate_literal(env, context, literal)
        }
        PengExpression::Identifier(identifier) => {
            generate_identifier(env, context, identifier)
        }
        PengExpression::Unary {
            operator,
            value,
        } => {
            match generate_expression(env, context, value) {
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
            match generate_expression(env, context, left) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match operator {
                PengBinaryOperator::ShortCircuitAnd
                | PengBinaryOperator::ShortCircuitOr => {
                    context.bytecode.push(PengInstruction::Duplicate(1));
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
                    context.bytecode.push(PengInstruction::Pop(1));

                    generate_expression(env, context, right)?;

                    let target = context.bytecode.len();
                    context.bytecode[jump_index] = match operator {
                        PengBinaryOperator::ShortCircuitAnd => {
                            PengInstruction::JumpIfFalse(target)
                        }
                        PengBinaryOperator::ShortCircuitOr => {
                            PengInstruction::JumpIfTrue(target)
                        }
                        _ => unreachable!(),
                    };
                }
                _ => {
                    generate_expression(env, context, right)?;
                    generate_binary_operator(context, operator);
                }
            }

            Ok(())
        }
        PengExpression::Type(type_expression) => {
            generate_type_expression(
                env,
                context,
                type_expression,
            )
        }
        PengExpression::FuncCall(call) => {
            generate_function_call(env, context, call)
        }
        PengExpression::MethodCall(_) => {
            todo!("generate method call expression")
        }
        PengExpression::AttributeAccess(_) => {
            todo!("generate attribute access expression")
        }
        PengExpression::MemberAccess(_) => {
            todo!("generate member access expression")
        }
        PengExpression::Index(index) => {
            generate_index_expression(env, context, index)
        }
        PengExpression::ObjectConstruction(_) => {
            todo!("generate object construction expression")
        }
        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => {
            generate_operation_call(
                env,
                context,
                left,
                operation,
                right,
            )
        }
        PengExpression::Try { .. } => {
            todo!("generate try expression")
        }
    }
}


pub fn generate_index_expression(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    index: &PengIndexExpression,
) -> Result<(), PengError> {
    match generate_expression(env, context, &index.object) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_expression(env, context, &index.index) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    context.bytecode.push(PengInstruction::GetIndex);
    Ok(())
}

pub fn generate_type_expression(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    type_expression: &PengPositionedTypeExpression,
) -> Result<(), PengError> {
    match &type_expression.value {
        PengTypeExpression::Union(types) => {
            for typ in types {
                match generate_type_expression(
                    env,
                    context,
                    typ,
                ) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            context.bytecode.push(
                PengInstruction::CreateUnion(types.len()),
            );
            Ok(())
        }
        PengTypeExpression::Custom(expression) => {
            generate_expression(env, context, expression)
        }
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

            context.push_const_and_const_instruction(
                PengValue::Type(
                    PengType::Vector(Box::new(inner_type)),
                ),
            );
            Ok(())
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
                PengValue::Type(typ),
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
        PengTypeExpression::Any => Some(PengType::Any),
        PengTypeExpression::UnionType => Some(PengType::Union),
        PengTypeExpression::Vector(inner) => {
            match static_type_from_expression(inner) {
                Some(inner_type) => {
                    Some(PengType::Vector(Box::new(inner_type)))
                }
                None => None,
            }
        }
        PengTypeExpression::Custom(_) => None,
        PengTypeExpression::Union(_) => None,
    }
}
