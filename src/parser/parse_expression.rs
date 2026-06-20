use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    parse_expression_bp(ptokens, 0)
}

fn parse_expression_bp(
    ptokens: &mut PengPeekablePositionedToken,
    min_bp: u8,
) -> Result<PengPositionedExpression, PengError> {
    let mut left = match parse_prefix_expression(ptokens) {
        Ok(expr) => expr,
        Err(e) => return Err(e),
    };

    loop {
        let op = match ptokens.peek() {
            Some(t) => match binary_operator_from_token(&t.value) {
                Some(op) => op,
                None => break,
            },
            None => break,
        };

        let (left_bp, right_bp) = binary_binding_power(&op);

        if left_bp < min_bp {
            break;
        }

        ptokens.next();

        let right = match parse_expression_bp(ptokens, right_bp) {
            Ok(expr) => expr,
            Err(e) => return Err(e),
        };

        let pos = left.position.clone();

        left = PengPositioned {
            value: PengExpression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
            position: pos,
        };
    }

    Ok(left)
}

fn parse_prefix_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let token = match ptokens.peek() {
        Some(t) => t.clone(),
        None => {
            return Err(PengError::new_message(
                "expected expression".to_string(),
            ));
        }
    };

    match &token.value {
        PengToken::Minus => {
            ptokens.next();

            let value = match parse_expression_bp(ptokens, 13) {
                Ok(expr) => expr,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                value: PengExpression::Unary {
                    operator: PengUnaryOperator::Negate,
                    value: Box::new(value),
                },
                position: token.position.clone(),
            })
        }

        PengToken::Exclamation => {
            ptokens.next();

            let value = match parse_expression_bp(ptokens, 13) {
                Ok(expr) => expr,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                value: PengExpression::Unary {
                    operator: PengUnaryOperator::Not,
                    value: Box::new(value),
                },
                position: token.position.clone(),
            })
        }

        _ => parse_postfix_expression(ptokens),
    }
}

fn parse_postfix_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let mut expr = match parse_primary_expression(ptokens) {
        Ok(expr) => expr,
        Err(e) => return Err(e),
    };

    loop {
        let token = match ptokens.peek() {
            Some(t) => t.clone(),
            None => break,
        };

        match &token.value {
            PengToken::LeftParenthesis => {
                return parse_func_call_expression(ptokens, expr);
            }

            PengToken::Dot => {
                return parse_dot_expression(ptokens, expr);
            }

            PengToken::Colon => {
                return parse_colon_func_call_expression(ptokens, expr);
            }

            PengToken::LeftBracket => {
                return parse_index_expression(ptokens, expr);
            }

            _ => break,
        }
    }

    Ok(expr)
}

fn parse_primary_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let token = match ptokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::new_message(
                "expected expression".to_string(),
            ));
        }
    };

    match &token.value {
        PengToken::Nil => literal_expr(PengLiteral::Nil, token.position.clone()),

        PengToken::IntLiteral(v) => literal_expr(PengLiteral::Int(*v), token.position.clone()),
        PengToken::UintLiteral(v) => literal_expr(PengLiteral::Uint(*v), token.position.clone()),
        PengToken::ByteLiteral(v) => literal_expr(PengLiteral::Byte(*v), token.position.clone()),

        PengToken::Float32Literal(v) => literal_expr(PengLiteral::Float32(*v), token.position.clone()),
        PengToken::Float64Literal(v) => literal_expr(PengLiteral::Float64(*v), token.position.clone()),

        PengToken::True => literal_expr(PengLiteral::Bool(true), token.position.clone()),
        PengToken::False => literal_expr(PengLiteral::Bool(false), token.position.clone()),

        PengToken::StringLiteral(s) => {
            literal_expr(PengLiteral::String(s.clone()), token.position.clone())
        }

        PengToken::Identifier(s) => {
            Ok(PengPositioned {
                value: PengExpression::Identifier(PengPositioned {
                    value: s.clone(),
                    position: token.position.clone(),
                }),
                position: token.position.clone(),
            })
        }

        PengToken::LeftParenthesis => {
            let expr = match parse_expression(ptokens) {
                Ok(expr) => expr,
                Err(e) => return Err(e),
            };

            let close = match ptokens.next() {
                Some(t) => t,
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected ')'".to_string(),
                        token.position.clone(),
                    ));
                }
            };

            match &close.value {
                PengToken::RightParenthesis => Ok(expr),
                _ => {
                    Err(PengError::new_positioned_message(
                        "expected ')'".to_string(),
                        close.position.clone(),
                    ))
                }
            }
        }

        _ => {
            Err(PengError::new_positioned_message(
                "expected expression".to_string(),
                token.position.clone(),
            ))
        }
    }
}

fn literal_expr(
    literal: PengLiteral,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    Ok(PengPositioned {
        value: PengExpression::Literal(PengPositioned {
            value: literal,
            position: position.clone(),
        }),
        position,
    })
}

fn binary_operator_from_token(token: &PengToken) -> Option<PengBinaryOperator> {
    match token {
        PengToken::Plus => Some(PengBinaryOperator::Add),
        PengToken::Minus => Some(PengBinaryOperator::Subtract),
        PengToken::Asterisk => Some(PengBinaryOperator::Multiply),
        PengToken::Slash => Some(PengBinaryOperator::Divide),
        PengToken::DoubleAsterisk => Some(PengBinaryOperator::Power),
        PengToken::Percent => Some(PengBinaryOperator::Remainder),

        PengToken::DoubleDot => Some(PengBinaryOperator::Concat),

        PengToken::DoubleAmpersand => Some(PengBinaryOperator::And),
        PengToken::DoublePipe => Some(PengBinaryOperator::Or),

        PengToken::DoubleEquals => Some(PengBinaryOperator::Equals),
        PengToken::ExclamationEquals => Some(PengBinaryOperator::NotEquals),
        PengToken::GreaterThan => Some(PengBinaryOperator::GreaterThan),
        PengToken::GreaterThanEquals => Some(PengBinaryOperator::GreaterEqualsThan),
        PengToken::LessThan => Some(PengBinaryOperator::LessThan),
        PengToken::LessThanEquals => Some(PengBinaryOperator::LessEqualsThan),

        PengToken::Is => Some(PengBinaryOperator::Is),
        PengToken::As => Some(PengBinaryOperator::As),

        PengToken::Equals => Some(PengBinaryOperator::Assign),

        PengToken::PlusEquals => Some(PengBinaryOperator::AddAssign),
        PengToken::MinusEquals => Some(PengBinaryOperator::SubtractAssign),
        PengToken::AsteriskEquals => Some(PengBinaryOperator::MultiplyAssign),
        PengToken::SlashEquals => Some(PengBinaryOperator::DivideAssign),
        PengToken::DoubleAsteriskEquals => Some(PengBinaryOperator::PowerAssign),
        PengToken::PercentEquals => Some(PengBinaryOperator::RemainderAssign),

        _ => None,
    }
}

fn binary_binding_power(op: &PengBinaryOperator) -> (u8, u8) {
    match op {
        PengBinaryOperator::Assign
        | PengBinaryOperator::AddAssign
        | PengBinaryOperator::SubtractAssign
        | PengBinaryOperator::MultiplyAssign
        | PengBinaryOperator::DivideAssign
        | PengBinaryOperator::PowerAssign
        | PengBinaryOperator::RemainderAssign => (1, 1),

        PengBinaryOperator::Or => (2, 3),
        PengBinaryOperator::And => (4, 5),

        PengBinaryOperator::Equals
        | PengBinaryOperator::NotEquals
        | PengBinaryOperator::GreaterThan
        | PengBinaryOperator::GreaterEqualsThan
        | PengBinaryOperator::LessThan
        | PengBinaryOperator::LessEqualsThan
        | PengBinaryOperator::Is
        | PengBinaryOperator::As => (6, 7),

        PengBinaryOperator::Concat => (8, 8),

        PengBinaryOperator::Add
        | PengBinaryOperator::Subtract => (9, 10),

        PengBinaryOperator::Multiply
        | PengBinaryOperator::Divide
        | PengBinaryOperator::Remainder => (11, 12),

        PengBinaryOperator::Power => (14, 13),
    }
}

fn parse_func_call_expression(
    _ptokens: &mut PengPeekablePositionedToken,
    _function: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    Err(PengError::new_message(
        "parse_func_call_expression not implemented".to_string(),
    ))
}

fn parse_dot_expression(
    _ptokens: &mut PengPeekablePositionedToken,
    _object: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    Err(PengError::new_message(
        "parse_dot_expression not implemented".to_string(),
    ))
}

fn parse_colon_func_call_expression(
    _ptokens: &mut PengPeekablePositionedToken,
    _module: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    Err(PengError::new_message(
        "parse_colon_func_call_expression not implemented".to_string(),
    ))
}

fn parse_index_expression(
    _ptokens: &mut PengPeekablePositionedToken,
    _object: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    Err(PengError::new_message(
        "parse_index_expression not implemented".to_string(),
    ))
}