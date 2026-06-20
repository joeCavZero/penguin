use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let first = match parse_single_type_expression(ptokens) {
        Ok(type_expression) => type_expression,
        Err(e) => return Err(e),
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

        let type_expression = match parse_single_type_expression(ptokens) {
            Ok(type_expression) => type_expression,
            Err(e) => return Err(e),
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

fn parse_single_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let mut type_expression = match parse_primary_type_expression(ptokens) {
        Ok(type_expression) => type_expression,
        Err(e) => return Err(e),
    };

    let is_nillable = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Question => true,
            _ => false,
        },
        None => false,
    };

    if is_nillable {
        ptokens.next();

        let position = type_expression.position.clone();

        type_expression = PengPositioned {
            value: PengTypeExpression::Nillable(Box::new(type_expression)),
            position,
        };
    }

    Ok(type_expression)
}

fn parse_primary_type_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    let token = match ptokens.peek() {
        Some(token) => (*token).clone(),
        None => {
            return Err(PengError::new_message(
                "expected type expression".to_string(),
            ));
        }
    };

    let is_type_literal = match &token.value {
        PengToken::Type => match token_after_type_token(ptokens) {
            Some(next) => match &next.value {
                PengToken::LessThan
                | PengToken::Colon
                | PengToken::LeftCurlyBrace => true,
                _ => false,
            },
            None => false,
        },
        _ => false,
    };

    if is_type_literal {
        let expression = match parse_type_literal(ptokens) {
            Ok(expression) => expression,
            Err(e) => return Err(e),
        };

        return Ok(PengPositioned {
            value: PengTypeExpression::Custom(Box::new(expression)),
            position: token.position.clone(),
        });
    }

    match &token.value {
        PengToken::Nil
        | PengToken::Int
        | PengToken::Uint
        | PengToken::Float32
        | PengToken::Float64
        | PengToken::Byte
        | PengToken::Bool
        | PengToken::String
        | PengToken::Any
        | PengToken::Type
        | PengToken::Mod
        | PengToken::Func
        | PengToken::Oper => {
            ptokens.next();

            let value = match type_expression_from_token(&token.value) {
                Some(value) => value,
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected type expression".to_string(),
                        token.position.clone(),
                    ));
                }
            };

            Ok(PengPositioned {
                value,
                position: token.position.clone(),
            })
        }

        PengToken::LeftBracket => {
            ptokens.next();

            let inner = match parse_type_expression(ptokens) {
                Ok(type_expression) => type_expression,
                Err(e) => return Err(e),
            };

            let close_token = match ptokens.next() {
                Some(token) => token,
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected ']'".to_string(),
                        token.position.clone(),
                    ));
                }
            };

            match &close_token.value {
                PengToken::RightBracket => {
                    Ok(PengPositioned {
                        value: PengTypeExpression::Vector(Box::new(inner)),
                        position: token.position.clone(),
                    })
                }
                _ => {
                    Err(PengError::new_positioned_message(
                        "expected ']'".to_string(),
                        close_token.position.clone(),
                    ))
                }
            }
        }

        _ => {
            let expression = match parse_expression(ptokens) {
                Ok(expression) => expression,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                value: PengTypeExpression::Custom(Box::new(expression)),
                position: token.position.clone(),
            })
        }
    }
}

pub(crate) fn type_expression_from_token(
    token: &PengToken,
) -> Option<PengTypeExpression> {
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
        PengToken::Oper => Some(PengTypeExpression::Operation),
        _ => None,
    }
}

fn token_after_type_token<'a>(
    ptokens: &PengPeekablePositionedToken<'a>,
) -> Option<&'a PengPositionedToken> {
    let mut lookahead = ptokens.clone();

    match lookahead.next() {
        Some(_) => {}
        None => return None,
    }

    match lookahead.next() {
        Some(token) => Some(token),
        None => None,
    }
}
