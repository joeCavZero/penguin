use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let first = match parse_primary_type_expression(ptokens) {
        Ok(type_expression) => type_expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_expression".to_string(),
            )));
        }
    };

    let position = first.position.clone();
    let mut types = vec![first];

    loop {
        let has_pipe = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::Pipe => true,
                _ => false,
            },
            None => false,
        };

        if !has_pipe {
            break;
        }

        ptokens.next();

        let type_expression = match parse_primary_type_expression(ptokens) {
            Ok(type_expression) => type_expression,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_type_expression".to_string(),
                )));
            }
        };

        types.push(type_expression);
    }

    if types.len() == 1 {
        match types.pop() {
            Some(type_expression) => Ok(type_expression),
            None => Err(PengError::new_positioned_message(
                "expected type expression".to_string(),
                position,
            )),
        }
    } else {
        Ok(PengPositioned {
            value: PengTypeExpression::Union(types),
            position,
        })
    }
}

fn parse_primary_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let token = match ptokens.peek() {
        Some(token) => (*token).clone(),
        None => {
            return Err(PengError::SyntaxError(
                "expected type expression".to_string(),
            ));
        }
    };

    match &token.value {
        PengToken::Nil => parse_builtin_type(ptokens, PengTypeExpression::Nil),
        PengToken::Int => parse_builtin_type(ptokens, PengTypeExpression::Int),
        PengToken::Uint => parse_builtin_type(ptokens, PengTypeExpression::Uint),
        PengToken::Float32 => parse_builtin_type(ptokens, PengTypeExpression::Float32),
        PengToken::Float64 => parse_builtin_type(ptokens, PengTypeExpression::Float64),
        PengToken::Byte => parse_builtin_type(ptokens, PengTypeExpression::Byte),
        PengToken::Bool => parse_builtin_type(ptokens, PengTypeExpression::Bool),
        PengToken::String => parse_builtin_type(ptokens, PengTypeExpression::String),
        PengToken::Any => parse_builtin_type(ptokens, PengTypeExpression::Any),

        PengToken::Type => parse_type_type_expression(ptokens),
        PengToken::Mod => parse_builtin_type(ptokens, PengTypeExpression::Module),
        PengToken::Func => parse_builtin_type(ptokens, PengTypeExpression::Function),
        PengToken::Oper => parse_builtin_type(ptokens, PengTypeExpression::Operation),
        PengToken::Thread => parse_builtin_type(ptokens, PengTypeExpression::Thread),

        PengToken::LeftBracket => parse_vector_type_expression(ptokens),

        PengToken::Union => parse_builtin_type(ptokens, PengTypeExpression::UnionType),

        PengToken::Identifier(_) => {
            let expression = match parse_custom_type_expression(ptokens) {
                Ok(expression) => expression,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_type_expression".to_string(),
                    )));
                }
            };

            match reject_invalid_custom_type_tail(ptokens) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_type_expression".to_string(),
                    )));
                }
            }

            Ok(PengPositioned {
                value: PengTypeExpression::Custom(Box::new(expression)),
                position: token.position.clone(),
            })
        }

        _ => Err(PengError::new_positioned_message(
            "expected type expression".to_string(),
            token.position.clone(),
        )),
    }
}

fn parse_builtin_type(
    ptokens: &mut PengPeekablePositionedToken,
    type_expression: PengTypeExpression,
) -> Result<PengPositionedTypeExpression, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected type expression".to_string(),
            ));
        }
    };

    Ok(PengPositioned {
        value: type_expression,
        position: token.position.clone(),
    })
}

fn parse_custom_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let first_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected custom type".to_string()));
        }
    };

    let first_name = match &first_token.value {
        PengToken::Identifier(name) => PengPositioned {
            value: name.clone(),
            position: first_token.position.clone(),
        },
        _ => {
            return Err(PengError::new_positioned_message(
                "expected custom type name".to_string(),
                first_token.position.clone(),
            ));
        }
    };

    let mut expression = PengPositioned {
        value: PengExpression::Identifier(first_name),
        position: first_token.position.clone(),
    };

    expression = match parse_custom_type_colon_chain(ptokens, expression) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_expression".to_string(),
            )));
        }
    };

    Ok(expression)
}

fn parse_custom_type_colon_chain(
    ptokens: &mut PengPeekablePositionedToken,
    left: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let has_colon = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Colon => true,
            _ => false,
        },
        None => false,
    };

    if !has_colon {
        return Ok(left);
    }

    ptokens.next();

    let name_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected type member name after ':'".to_string(),
                left.position.clone(),
            ));
        }
    };

    let name = match &name_token.value {
        PengToken::Identifier(name) => PengPositioned {
            value: name.clone(),
            position: name_token.position.clone(),
        },
        _ => {
            return Err(PengError::new_positioned_message(
                "expected type member name after ':'".to_string(),
                name_token.position.clone(),
            ));
        }
    };

    let expression = PengPositioned {
        value: PengExpression::MemberAccess(PengMemberAccessExpression {
            object: Box::new(left),
            name,
        }),
        position: name_token.position.clone(),
    };

    let has_another_colon = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Colon => true,
            _ => false,
        },
        None => false,
    };

    if has_another_colon {
        return parse_custom_type_colon_chain(ptokens, expression);
    }

    Ok(expression)
}

fn reject_invalid_custom_type_tail(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<(), PengError> {
    let token = match ptokens.peek() {
        Some(token) => token,
        None => return Ok(()),
    };

    match &token.value {
        PengToken::Dot => Err(PengError::new_positioned_message(
            "'.' is not allowed in type expressions; use ':' for module type access".to_string(),
            token.position.clone(),
        )),
        PengToken::LeftParenthesis => Err(PengError::new_positioned_message(
            "function call is not allowed in type expressions".to_string(),
            token.position.clone(),
        )),
        _ => Ok(()),
    }
}

pub fn type_expression_from_token(token: &PengToken) -> Option<PengTypeExpression> {
    match token {
        PengToken::Nil => Some(PengTypeExpression::Nil),
        PengToken::Int => Some(PengTypeExpression::Int),
        PengToken::Uint => Some(PengTypeExpression::Uint),
        PengToken::Float32 => Some(PengTypeExpression::Float32),
        PengToken::Float64 => Some(PengTypeExpression::Float64),
        PengToken::Byte => Some(PengTypeExpression::Byte),
        PengToken::Bool => Some(PengTypeExpression::Bool),
        PengToken::String => Some(PengTypeExpression::String),
        PengToken::Any => Some(PengTypeExpression::Any),
        PengToken::Type => Some(PengTypeExpression::Type),
        PengToken::Mod => Some(PengTypeExpression::Module),
        PengToken::Func => Some(PengTypeExpression::Function),
        PengToken::Thread => Some(PengTypeExpression::Thread),
        PengToken::Oper => Some(PengTypeExpression::Operation),
        PengToken::Union => Some(PengTypeExpression::Union(Vec::new())),
        _ => None,
    }
}

fn parse_vector_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected '['".to_string()));
        }
    };

    let inner = match parse_type_expression(ptokens) {
        Ok(type_expression) => type_expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_expression".to_string(),
            )));
        }
    };

    let close_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected ']'".to_string(),
                open_token.position.clone(),
            ));
        }
    };

    match &close_token.value {
        PengToken::RightBracket => Ok(PengPositioned {
            value: PengTypeExpression::Vector(Box::new(inner)),
            position: open_token.position.clone(),
        }),
        _ => Err(PengError::new_positioned_message(
            "expected ']'".to_string(),
            close_token.position.clone(),
        )),
    }
}

fn parse_type_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let type_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected 'type'".to_string()));
        }
    };

    match &type_token.value {
        PengToken::Type => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'type'".to_string(),
                type_token.position.clone(),
            ));
        }
    }

    let is_literal_shape = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LessThan | PengToken::LeftCurlyBrace => true,
            _ => false,
        },
        None => false,
    };

    if !is_literal_shape {
        return Ok(PengPositioned {
            value: PengTypeExpression::Type,
            position: type_token.position.clone(),
        });
    }

    let supers = Vec::new();

    let (fields, functions) = match parse_type_members(ptokens) {
        Ok(members) => members,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_expression".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        value: PengTypeExpression::Custom(Box::new(PengPositioned {
            value: PengExpression::Literal(PengPositioned {
                value: PengLiteral::Type(PengTypeLiteral {
                    supers,
                    fields,
                    functions,
                }),
                position: type_token.position.clone(),
            }),
            position: type_token.position.clone(),
        })),
        position: type_token.position.clone(),
    })
}
