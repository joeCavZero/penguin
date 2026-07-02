use crate::core::*;
use crate::parser::*;

pub fn simplify_expression(
    expression: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let position = expression.position.clone();

    match expression.value {
        PengExpression::Unary { operator, value } => simplify_unary(operator, *value, position),

        PengExpression::Binary {
            left,
            operator,
            right,
        } => simplify_binary(*left, operator, *right, position),

        value => Ok(PengPositioned { position, value }),
    }
}

fn simplify_unary(
    operator: PengUnaryOperator,
    value: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match operator {
        PengUnaryOperator::Not => {
            match bool_literal_value(&value) {
                Some(v) => {
                    return Ok(bool_literal_expression(!v, position));
                }

                None => {}
            }

            match value.value {
                PengExpression::Unary {
                    operator: PengUnaryOperator::Not,
                    value: inner,
                } => {
                    if is_known_bool_expression(&inner.value) {
                        return Ok(*inner);
                    }

                    Ok(PengPositioned {
                        position,
                        value: PengExpression::Unary {
                            operator: PengUnaryOperator::Not,
                            value: Box::new(PengPositioned {
                                position: value.position,
                                value: PengExpression::Unary {
                                    operator: PengUnaryOperator::Not,
                                    value: inner,
                                },
                            }),
                        },
                    })
                }

                value => Ok(PengPositioned {
                    position: position.clone(),
                    value: PengExpression::Unary {
                        operator: PengUnaryOperator::Not,
                        value: Box::new(PengPositioned {
                            position: position.clone(),
                            value,
                        }),
                    },
                }),
            }
        }

        PengUnaryOperator::Negate => Ok(PengPositioned {
            position,
            value: PengExpression::Unary {
                operator: PengUnaryOperator::Negate,
                value: Box::new(value),
            },
        }),
    }
}

fn simplify_binary(
    left: PengPositionedExpression,
    operator: PengBinaryOperator,
    right: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match operator {
        PengBinaryOperator::ShortCircuitAnd => simplify_short_circuit_and(left, right, position),

        PengBinaryOperator::ShortCircuitOr => simplify_short_circuit_or(left, right, position),

        PengBinaryOperator::NonShortCircuitAnd => {
            simplify_non_short_circuit_and(left, right, position)
        }

        PengBinaryOperator::NonShortCircuitOr => {
            simplify_non_short_circuit_or(left, right, position)
        }

        PengBinaryOperator::Equals => simplify_bool_equals(left, right, position, false),

        PengBinaryOperator::NotEquals => simplify_bool_equals(left, right, position, true),

        _ => Ok(PengPositioned {
            position,
            value: PengExpression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
        }),
    }
}

fn simplify_short_circuit_and(
    left: PengPositionedExpression,
    right: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match bool_literal_value(&left) {
        Some(false) => {
            return Ok(bool_literal_expression(false, position));
        }

        Some(true) => {
            if is_known_bool_expression(&right.value) {
                return Ok(right);
            }
        }

        None => {}
    }

    match bool_literal_value(&right) {
        Some(true) => {
            if is_known_bool_expression(&left.value) {
                return Ok(left);
            }
        }

        _ => {}
    }

    Ok(PengPositioned {
        position,
        value: PengExpression::Binary {
            left: Box::new(left),
            operator: PengBinaryOperator::ShortCircuitAnd,
            right: Box::new(right),
        },
    })
}

fn simplify_short_circuit_or(
    left: PengPositionedExpression,
    right: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match bool_literal_value(&left) {
        Some(true) => {
            return Ok(bool_literal_expression(true, position));
        }

        Some(false) => {
            if is_known_bool_expression(&right.value) {
                return Ok(right);
            }
        }

        None => {}
    }

    match bool_literal_value(&right) {
        Some(false) => {
            if is_known_bool_expression(&left.value) {
                return Ok(left);
            }
        }

        _ => {}
    }

    Ok(PengPositioned {
        position,
        value: PengExpression::Binary {
            left: Box::new(left),
            operator: PengBinaryOperator::ShortCircuitOr,
            right: Box::new(right),
        },
    })
}

fn simplify_non_short_circuit_and(
    left: PengPositionedExpression,
    right: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match bool_literal_value(&left) {
        Some(true) => {
            if is_known_bool_expression(&right.value) {
                return Ok(right);
            }
        }

        _ => {}
    }

    match bool_literal_value(&right) {
        Some(true) => {
            if is_known_bool_expression(&left.value) {
                return Ok(left);
            }
        }

        _ => {}
    }

    Ok(PengPositioned {
        position,
        value: PengExpression::Binary {
            left: Box::new(left),
            operator: PengBinaryOperator::NonShortCircuitAnd,
            right: Box::new(right),
        },
    })
}

fn simplify_non_short_circuit_or(
    left: PengPositionedExpression,
    right: PengPositionedExpression,
    position: PengPosition,
) -> Result<PengPositionedExpression, PengError> {
    match bool_literal_value(&left) {
        Some(false) => {
            if is_known_bool_expression(&right.value) {
                return Ok(right);
            }
        }

        _ => {}
    }

    match bool_literal_value(&right) {
        Some(false) => {
            if is_known_bool_expression(&left.value) {
                return Ok(left);
            }
        }

        _ => {}
    }

    Ok(PengPositioned {
        position,
        value: PengExpression::Binary {
            left: Box::new(left),
            operator: PengBinaryOperator::NonShortCircuitOr,
            right: Box::new(right),
        },
    })
}

fn simplify_bool_equals(
    left: PengPositionedExpression,
    right: PengPositionedExpression,
    position: PengPosition,
    invert: bool,
) -> Result<PengPositionedExpression, PengError> {
    match bool_literal_value(&left) {
        Some(v) => {
            if is_known_bool_expression(&right.value) {
                if v {
                    if invert {
                        return Ok(not_expression(right, position));
                    }

                    return Ok(right);
                }

                if invert {
                    return Ok(right);
                }

                return Ok(not_expression(right, position));
            }
        }

        None => {}
    }

    match bool_literal_value(&right) {
        Some(v) => {
            if is_known_bool_expression(&left.value) {
                if v {
                    if invert {
                        return Ok(not_expression(left, position));
                    }

                    return Ok(left);
                }

                if invert {
                    return Ok(left);
                }

                return Ok(not_expression(left, position));
            }
        }

        None => {}
    }

    let operator = if invert {
        PengBinaryOperator::NotEquals
    } else {
        PengBinaryOperator::Equals
    };

    Ok(PengPositioned {
        position,
        value: PengExpression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        },
    })
}

fn bool_literal_value(expression: &PengPositionedExpression) -> Option<bool> {
    match &expression.value {
        PengExpression::Literal(literal) => match &literal.value {
            PengLiteral::Bool(v) => Some(*v),
            _ => None,
        },

        _ => None,
    }
}

fn bool_literal_expression(value: bool, position: PengPosition) -> PengPositionedExpression {
    PengPositioned {
        position: position.clone(),
        value: PengExpression::Literal(PengPositioned {
            position,
            value: PengLiteral::Bool(value),
        }),
    }
}

fn not_expression(
    expression: PengPositionedExpression,
    position: PengPosition,
) -> PengPositionedExpression {
    PengPositioned {
        position,
        value: PengExpression::Unary {
            operator: PengUnaryOperator::Not,
            value: Box::new(expression),
        },
    }
}

fn is_known_bool_expression(expression: &PengExpression) -> bool {
    match expression {
        PengExpression::Literal(literal) => match &literal.value {
            PengLiteral::Bool(_) => true,
            _ => false,
        },

        PengExpression::Unary {
            operator: PengUnaryOperator::Not,
            ..
        } => true,

        PengExpression::Binary { operator, .. } => match operator {
            PengBinaryOperator::ShortCircuitAnd
            | PengBinaryOperator::ShortCircuitOr
            | PengBinaryOperator::NonShortCircuitAnd
            | PengBinaryOperator::NonShortCircuitOr
            | PengBinaryOperator::Equals
            | PengBinaryOperator::NotEquals
            | PengBinaryOperator::GreaterThan
            | PengBinaryOperator::GreaterEqualsThan
            | PengBinaryOperator::LessThan
            | PengBinaryOperator::LessEqualsThan => true,

            _ => false,
        },

        _ => false,
    }
}
