use crate::core::*;
use crate::lexer::*;
use crate::parser::parse_function_literal::*;
use crate::parser::parse_function_params::*;
use crate::parser::parse_module_literal::*;
use crate::parser::parse_object_literal::*;
use crate::parser::parse_operation_literal::*;
use crate::parser::parse_type_expression::*;
use crate::parser::parse_type_literal::*;
use crate::parser::parse_vector_literal::*;
use crate::parser::parser::*;

pub fn parse_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    parse_expression_bp(ptokens, 0, true)
}

pub fn parse_expression_before_as(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    parse_expression_bp(ptokens, 0, false)
}

fn parse_expression_bp(
    ptokens: &mut PengPeekablePositionedToken,
    min_bp: u8,
    allow_as: bool,
) -> Result<PengPositionedExpression, PengError> {
    let mut left = match parse_prefix_expression(ptokens) {
        Ok(expr) => expr,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    loop {
        let has_operation = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::Identifier(_) | PengToken::Oper => {
                    positions_share_line(&left.position, &token.position)
                }
                _ => false,
            },
            None => false,
        };

        if has_operation {
            let operation_bp = 7;

            if operation_bp < min_bp {
                break;
            }

            let operation = match parse_infix_operation_expression(ptokens) {
                Ok(operation) => operation,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
            };

            let right = match parse_expression_bp(ptokens, operation_bp + 1, allow_as) {
                Ok(expression) => expression,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
            };

            let position = left.position.clone();

            left = PengPositioned {
                value: PengExpression::OperationCall {
                    left: Box::new(left),
                    operation: Box::new(operation),
                    right: Box::new(right),
                },
                position,
            };

            continue;
        }

        let operator_token = match ptokens.peek() {
            Some(token) => (*token).clone(),
            None => break,
        };

        let op = match binary_operator_from_token(&operator_token.value) {
            Some(op) => op,
            None => break,
        };

        let is_as = match &op {
            PengBinaryOperator::As => true,
            _ => false,
        };

        if is_as && !allow_as {
            break;
        }

        let (left_bp, right_bp) = binary_binding_power(&op);

        if left_bp < min_bp {
            break;
        }

        ptokens.next();

        let right = match parse_expression_bp(ptokens, right_bp, allow_as) {
            Ok(expr) => expr,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_expression".to_string(),
                )));
            }
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
        Some(t) => *t,
        None => {
            return Err(PengError::SyntaxError("expected expression".to_string()));
        }
    };

    match &token.value {
        PengToken::Minus => {
            ptokens.next();

            let value = match parse_expression_bp(ptokens, 13, true) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
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

            let value = match parse_expression_bp(ptokens, 13, true) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
            };

            Ok(PengPositioned {
                value: PengExpression::Unary {
                    operator: PengUnaryOperator::Not,
                    value: Box::new(value),
                },
                position: token.position.clone(),
            })
        }

        PengToken::Try => {
            ptokens.next();

            let value = match parse_expression_bp(ptokens, 0, true) {
                Ok(expression) => expression,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
            };

            let has_else = match ptokens.peek() {
                Some(next) => match &next.value {
                    PengToken::Else => true,
                    _ => false,
                },
                None => false,
            };

            let elsing = if has_else {
                ptokens.next();

                match parse_expression_bp(ptokens, 0, true) {
                    Ok(expression) => Some(Box::new(expression)),
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_expression".to_string(),
                        )));
                    }
                }
            } else {
                None
            };

            Ok(PengPositioned {
                value: PengExpression::Try {
                    value: Box::new(value),
                    elsing,
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
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    loop {
        let token = match ptokens.peek() {
            Some(t) => *t,
            None => break,
        };

        if is_type_value_expression(&expr.value)
            && !positions_share_line(&expr.position, &token.position)
        {
            break;
        }

        match &token.value {
            PengToken::LeftParenthesis => {
                expr = match parse_func_call_expression(ptokens, expr) {
                    Ok(expression) => expression,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_expression".to_string(),
                        )));
                    }
                };
            }

            PengToken::Dot => {
                expr = match parse_dot_expression(ptokens, expr) {
                    Ok(expression) => expression,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_expression".to_string(),
                        )));
                    }
                };
            }

            PengToken::Colon => {
                expr = match parse_colon_func_call_expression(ptokens, expr) {
                    Ok(expression) => expression,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_expression".to_string(),
                        )));
                    }
                };
            }

            PengToken::LeftBracket => {
                expr = match parse_index_expression(ptokens, expr) {
                    Ok(expression) => expression,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_expression".to_string(),
                        )));
                    }
                };
            }

            _ => break,
        }
    }

    Ok(expr)
}

fn parse_primary_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let is_function_literal = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Func => match token_after_current(ptokens) {
                Some(token) => match &token.value {
                    PengToken::LeftParenthesis => true,
                    _ => false,
                },
                None => false,
            },
            _ => false,
        },
        None => false,
    };

    if is_function_literal {
        return parse_function_literal(ptokens);
    }

    let primary_token = match ptokens.peek() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected expression".to_string()));
        }
    };

    match &primary_token.value {
        PengToken::Type => {
            let is_literal = match token_after_current(ptokens) {
                Some(token) => match &token.value {
                    PengToken::Colon | PengToken::LeftCurlyBrace => true,
                    _ => false,
                },
                None => false,
            };

            if is_literal {
                return parse_type_literal(ptokens);
            }

            return parse_type_value_expression(ptokens);
        }
        PengToken::Mod => {
            let is_literal = match token_after_current(ptokens) {
                Some(token) => match &token.value {
                    PengToken::LeftCurlyBrace => true,
                    _ => false,
                },
                None => false,
            };

            if is_literal {
                return parse_module_literal(ptokens);
            }
        }
        PengToken::LeftBracket => return parse_vector_literal(ptokens),
        PengToken::LeftCurlyBrace => return parse_object_literal(ptokens),
        PengToken::Oper => {
            let is_literal = match token_after_current(ptokens) {
                Some(token) => match &token.value {
                    PengToken::LeftParenthesis => true,
                    _ => false,
                },
                None => false,
            };

            if is_literal {
                return parse_operation_literal(ptokens);
            }
        }
        _ => {}
    }

    if let Some(expression) = parse_bare_type_value_expression(ptokens) {
        return Ok(expression);
    }

    let token = match ptokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::SyntaxError("expected expression".to_string()));
        }
    };

    match &token.value {
        PengToken::Nil => literal_expr(PengLiteral::Nil, token.position.clone()),

        PengToken::IntLiteral(v) => literal_expr(PengLiteral::Int(*v), token.position.clone()),
        PengToken::UintLiteral(v) => literal_expr(PengLiteral::Uint(*v), token.position.clone()),
        PengToken::ByteLiteral(v) => literal_expr(PengLiteral::Byte(*v), token.position.clone()),

        PengToken::Float32Literal(v) => {
            literal_expr(PengLiteral::Float32(*v), token.position.clone())
        }
        PengToken::Float64Literal(v) => {
            literal_expr(PengLiteral::Float64(*v), token.position.clone())
        }

        PengToken::True => literal_expr(PengLiteral::Bool(true), token.position.clone()),
        PengToken::False => literal_expr(PengLiteral::Bool(false), token.position.clone()),

        PengToken::StringLiteral(s) => {
            literal_expr(PengLiteral::String(s.clone()), token.position.clone())
        }

        // TODO: Custom type names and value names share the Identifier token. Enforce the
        // `type Name` rule for custom types during name resolution, where the binding kind is
        // known; the parser cannot reject a bare custom type without rejecting normal values.
        PengToken::Identifier(s) => Ok(PengPositioned {
            value: PengExpression::Identifier(PengPositioned {
                value: s.clone(),
                position: token.position.clone(),
            }),
            position: token.position.clone(),
        }),

        PengToken::LeftParenthesis => {
            let expr = match parse_expression(ptokens) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_expression".to_string(),
                    )));
                }
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
                _ => Err(PengError::new_positioned_message(
                    "expected ')'".to_string(),
                    close.position.clone(),
                )),
            }
        }

        _ => Err(PengError::new_positioned_message(
            "expected expression".to_string(),
            token.position.clone(),
        )),
    }
}

fn parse_bare_type_value_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Option<PengPositionedExpression> {
    let token = match ptokens.peek() {
        Some(token) => (*token).clone(),
        None => return None,
    };

    let type_expression = match &token.value {
        PengToken::Int => PengTypeExpression::Int,
        PengToken::Uint => PengTypeExpression::Uint,
        PengToken::Float32 => PengTypeExpression::Float32,
        PengToken::Float64 => PengTypeExpression::Float64,
        PengToken::Byte => PengTypeExpression::Byte,
        PengToken::Bool => PengTypeExpression::Bool,
        PengToken::String => PengTypeExpression::String,
        PengToken::Any => PengTypeExpression::Any,
        PengToken::Mod => PengTypeExpression::Module,
        PengToken::Func => PengTypeExpression::Function,
        PengToken::Oper => PengTypeExpression::Operation,
        PengToken::Thread => PengTypeExpression::Thread,
        _ => return None,
    };

    ptokens.next();

    Some(PengPositioned {
        value: PengExpression::Type(PengPositioned {
            value: type_expression,
            position: token.position.clone(),
        }),
        position: token.position.clone(),
    })
}

fn parse_type_value_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
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

    let type_expression = match parse_type_expression(ptokens) {
        Ok(type_expression) => type_expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing type value expression".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        value: PengExpression::Type(type_expression),
        position: type_token.position.clone(),
    })
}

pub fn literal_expr(
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

        PengToken::Ampersand => Some(PengBinaryOperator::NonShortCircuitAnd),
        PengToken::DoubleAmpersand => Some(PengBinaryOperator::ShortCircuitAnd),
        PengToken::Pipe => Some(PengBinaryOperator::NonShortCircuitOr),
        PengToken::DoublePipe => Some(PengBinaryOperator::ShortCircuitOr),

        PengToken::DoubleEquals => Some(PengBinaryOperator::Equals),
        PengToken::ExclamationEquals => Some(PengBinaryOperator::NotEquals),
        PengToken::GreaterThan => Some(PengBinaryOperator::GreaterThan),
        PengToken::GreaterThanEquals => Some(PengBinaryOperator::GreaterEqualsThan),
        PengToken::LessThan => Some(PengBinaryOperator::LessThan),
        PengToken::LessThanEquals => Some(PengBinaryOperator::LessEqualsThan),

        PengToken::As => Some(PengBinaryOperator::As),

        _ => None,
    }
}

fn binary_binding_power(op: &PengBinaryOperator) -> (u8, u8) {
    match op {
        PengBinaryOperator::ShortCircuitOr | PengBinaryOperator::NonShortCircuitOr => (2, 3),
        PengBinaryOperator::ShortCircuitAnd | PengBinaryOperator::NonShortCircuitAnd => (4, 5),

        PengBinaryOperator::Equals
        | PengBinaryOperator::NotEquals
        | PengBinaryOperator::GreaterThan
        | PengBinaryOperator::GreaterEqualsThan
        | PengBinaryOperator::LessThan
        | PengBinaryOperator::LessEqualsThan => (6, 7),

        PengBinaryOperator::Concat => (8, 8),

        PengBinaryOperator::Add | PengBinaryOperator::Subtract => (9, 10),

        PengBinaryOperator::Multiply
        | PengBinaryOperator::Divide
        | PengBinaryOperator::Remainder => (11, 12),

        PengBinaryOperator::Power => (14, 13),

        PengBinaryOperator::As => (15, 16),
    }
}

fn parse_func_call_expression(
    ptokens: &mut PengPeekablePositionedToken,
    function: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let position = function.position.clone();
    let args = match parse_function_params(ptokens) {
        Ok(args) => args,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        value: PengExpression::FuncCall(PengFuncCallExpression {
            function: Box::new(function),
            args,
        }),
        position,
    })
}

fn parse_dot_expression(
    ptokens: &mut PengPeekablePositionedToken,
    object: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let dot_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected '.'".to_string()));
        }
    };

    let name = match crate::parser::parser_utils::expect_identifier(
        ptokens,
        "expected attribute name".to_string(),
        dot_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    let position = object.position.clone();
    let has_call = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftParenthesis => true,
            _ => false,
        },
        None => false,
    };

    if has_call {
        let args = match parse_function_params(ptokens) {
            Ok(args) => args,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_expression".to_string(),
                )));
            }
        };

        Ok(PengPositioned {
            value: PengExpression::MethodCall(PengMethodCallExpression {
                object: Box::new(object),
                method: name,
                args,
            }),
            position,
        })
    } else {
        Ok(PengPositioned {
            value: PengExpression::AttributeAccess(PengAttributeAccessExpression {
                object: Box::new(object),
                name,
            }),
            position,
        })
    }
}

fn parse_colon_func_call_expression(
    ptokens: &mut PengPeekablePositionedToken,
    module: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let colon_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected ':'".to_string()));
        }
    };

    let is_object = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftCurlyBrace | PengToken::LessThan => true,
            _ => false,
        },
        None => false,
    };

    if is_object {
        let open_token = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected '{'".to_string(),
                    colon_token.position.clone(),
                ));
            }
        };

        let fields = match parse_object_fields(ptokens, open_token.position.clone()) {
            Ok(fields) => fields,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_expression".to_string(),
                )));
            }
        };

        let position = module.position.clone();

        return Ok(PengPositioned {
            value: PengExpression::ObjectConstruction(PengObjectConstructionExpression {
                object_type: Box::new(module),
                fields,
            }),
            position,
        });
    }

    let name = match crate::parser::parser_utils::expect_identifier(
        ptokens,
        "expected module function name".to_string(),
        colon_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    let position = module.position.clone();
    let member = PengPositioned {
        value: PengExpression::MemberAccess(PengMemberAccessExpression {
            object: Box::new(module),
            name,
        }),
        position: position.clone(),
    };

    let has_call = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftParenthesis => true,
            _ => false,
        },
        None => false,
    };

    if !has_call {
        return Ok(member);
    }

    let args = match parse_function_params(ptokens) {
        Ok(args) => args,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        value: PengExpression::FuncCall(PengFuncCallExpression {
            function: Box::new(member),
            args,
        }),
        position,
    })
}

fn parse_index_expression(
    ptokens: &mut PengPeekablePositionedToken,
    object: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected '['".to_string()));
        }
    };

    let index = match parse_expression(ptokens) {
        Ok(expression) => expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_expression".to_string(),
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
        PengToken::RightBracket => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected ']'".to_string(),
                close_token.position.clone(),
            ));
        }
    }

    let position = object.position.clone();

    Ok(PengPositioned {
        value: PengExpression::Index(PengIndexExpression {
            object: Box::new(object),
            index: Box::new(index),
        }),
        position,
    })
}

fn parse_infix_operation_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let is_literal = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Oper => true,
            _ => false,
        },
        None => false,
    };

    if is_literal {
        return parse_operation_literal(ptokens);
    }

    let first_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected operation name".to_string(),
            ));
        }
    };

    let name = match &first_token.value {
        PengToken::Identifier(name) => PengPositioned {
            value: name.clone(),
            position: first_token.position.clone(),
        },
        _ => {
            return Err(PengError::new_positioned_message(
                "expected operation name".to_string(),
                first_token.position.clone(),
            ));
        }
    };

    let mut operation = PengPositioned {
        value: PengExpression::Identifier(name),
        position: first_token.position.clone(),
    };

    loop {
        let has_colon = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::Colon => true,
                _ => false,
            },
            None => false,
        };

        if !has_colon {
            break;
        }

        ptokens.next();

        let part = match crate::parser::parser_utils::expect_identifier(
            ptokens,
            "expected operation name after ':'".to_string(),
            first_token.position.clone(),
        ) {
            Ok(part) => part,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_expression".to_string(),
                )));
            }
        };

        let position = operation.position.clone();

        operation = PengPositioned {
            value: PengExpression::MemberAccess(PengMemberAccessExpression {
                object: Box::new(operation),
                name: part,
            }),
            position,
        };
    }

    Ok(operation)
}

fn is_type_value_expression(expression: &PengExpression) -> bool {
    match expression {
        PengExpression::Type(_) => true,
        PengExpression::Literal(literal) => match &literal.value {
            PengLiteral::Type(_) => true,
            _ => false,
        },
        _ => false,
    }
}

fn positions_share_line(left: &PengPosition, right: &PengPosition) -> bool {
    left.line == right.line
}

fn token_after_current<'a>(
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
