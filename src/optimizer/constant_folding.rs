use crate::core::*;
use crate::parser::*;

pub fn fold_expression(
    expression: PengPositionedExpression,
) -> Result<PengPositionedExpression, PengError> {
    let folded = match &expression.value {
        PengExpression::Unary { operator, value } => {
            let literal = match literal_from_expression(value) {
                Some(v) => v,
                None => return Ok(expression),
            };

            fold_unary(operator, literal)
        }

        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            let left_literal = match literal_from_expression(left) {
                Some(v) => v,
                None => return Ok(expression),
            };

            let right_literal = match literal_from_expression(right) {
                Some(v) => v,
                None => return Ok(expression),
            };

            fold_binary(operator, left_literal, right_literal)
        }

        _ => None,
    };

    match folded {
        Some(literal) => Ok(literal_expression(literal, expression.position)),
        None => Ok(expression),
    }
}

fn literal_from_expression(expression: &PengPositionedExpression) -> Option<&PengLiteral> {
    match &expression.value {
        PengExpression::Literal(literal) => Some(&literal.value),
        _ => None,
    }
}

fn literal_expression(
    literal: PengLiteral,
    position: PengPosition,
) -> PengPositionedExpression {
    PengPositioned {
        position: position.clone(),
        value: PengExpression::Literal(PengPositioned {
            position,
            value: literal,
        }),
    }
}

fn fold_unary(
    operator: &PengUnaryOperator,
    value: &PengLiteral,
) -> Option<PengLiteral> {
    match operator {
        PengUnaryOperator::Not => {
            match value {
                PengLiteral::Bool(v) => Some(PengLiteral::Bool(!v)),
                _ => None,
            }
        }

        PengUnaryOperator::Negate => {
            match value {
                PengLiteral::Int(v) => {
                    match v.checked_neg() {
                        Some(v) => Some(PengLiteral::Int(v)),
                        None => None,
                    }
                }

                PengLiteral::Float32(v) => Some(PengLiteral::Float32(-v)),
                PengLiteral::Float64(v) => Some(PengLiteral::Float64(-v)),

                _ => None,
            }
        }
    }
}

fn fold_binary(
    operator: &PengBinaryOperator,
    left: &PengLiteral,
    right: &PengLiteral,
) -> Option<PengLiteral> {
    match operator {
        PengBinaryOperator::Add => fold_add(left, right),
        PengBinaryOperator::Subtract => fold_subtract(left, right),
        PengBinaryOperator::Multiply => fold_multiply(left, right),
        PengBinaryOperator::Divide => fold_divide(left, right),
        PengBinaryOperator::Power => fold_power(left, right),
        PengBinaryOperator::Remainder => fold_remainder(left, right),

        PengBinaryOperator::Concat => fold_concat(left, right),

        PengBinaryOperator::ShortCircuitAnd
        | PengBinaryOperator::NonShortCircuitAnd => fold_and(left, right),

        PengBinaryOperator::ShortCircuitOr
        | PengBinaryOperator::NonShortCircuitOr => fold_or(left, right),

        PengBinaryOperator::Equals => fold_equals(left, right),
        PengBinaryOperator::NotEquals => fold_not_equals(left, right),

        PengBinaryOperator::GreaterThan => fold_greater_than(left, right),
        PengBinaryOperator::GreaterEqualsThan => fold_greater_equals_than(left, right),
        PengBinaryOperator::LessThan => fold_less_than(left, right),
        PengBinaryOperator::LessEqualsThan => fold_less_equals_than(left, right),

        PengBinaryOperator::As => None,
    }
}

fn fold_add(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            match a.checked_add(*b) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            match a.checked_add(*b) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            match a.checked_add(*b) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(*a + *b))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(*a + *b))
        }

        _ => None,
    }
}

fn fold_subtract(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            match a.checked_sub(*b) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            match a.checked_sub(*b) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            match a.checked_sub(*b) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(*a - *b))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(*a - *b))
        }

        _ => None,
    }
}

fn fold_multiply(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            match a.checked_mul(*b) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            match a.checked_mul(*b) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            match a.checked_mul(*b) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(*a * *b))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(*a * *b))
        }

        _ => None,
    }
}

fn fold_divide(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(_), PengLiteral::Int(0)) => None,
        (PengLiteral::Uint(_), PengLiteral::Uint(0)) => None,
        (PengLiteral::Byte(_), PengLiteral::Byte(0)) => None,
        (PengLiteral::Float32(_), PengLiteral::Float32(v)) if *v == 0.0 => None,
        (PengLiteral::Float64(_), PengLiteral::Float64(v)) if *v == 0.0 => None,

        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            match a.checked_div(*b) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            match a.checked_div(*b) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            match a.checked_div(*b) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(*a / *b))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(*a / *b))
        }

        _ => None,
    }
}

fn fold_remainder(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(_), PengLiteral::Int(0)) => None,
        (PengLiteral::Uint(_), PengLiteral::Uint(0)) => None,
        (PengLiteral::Byte(_), PengLiteral::Byte(0)) => None,
        (PengLiteral::Float32(_), PengLiteral::Float32(v)) if *v == 0.0 => None,
        (PengLiteral::Float64(_), PengLiteral::Float64(v)) if *v == 0.0 => None,

        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            match a.checked_rem(*b) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            match a.checked_rem(*b) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            match a.checked_rem(*b) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(*a % *b))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(*a % *b))
        }

        _ => None,
    }
}

fn fold_power(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => {
            let exp = match u32::try_from(*b) {
                Ok(v) => v,
                Err(_) => return None,
            };

            match a.checked_pow(exp) {
                Some(v) => Some(PengLiteral::Int(v)),
                None => None,
            }
        }

        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => {
            let exp = match u32::try_from(*b) {
                Ok(v) => v,
                Err(_) => return None,
            };

            match a.checked_pow(exp) {
                Some(v) => Some(PengLiteral::Uint(v)),
                None => None,
            }
        }

        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => {
            let exp = u32::from(*b);

            match a.checked_pow(exp) {
                Some(v) => Some(PengLiteral::Byte(v)),
                None => None,
            }
        }

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => {
            Some(PengLiteral::Float32(a.powf(*b)))
        }

        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => {
            Some(PengLiteral::Float64(a.powf(*b)))
        }

        _ => None,
    }
}

fn fold_concat(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::String(a), PengLiteral::String(b)) => {
            let mut value = String::new();
            value.push_str(a);
            value.push_str(b);

            Some(PengLiteral::String(value))
        }

        _ => None,
    }
}

fn fold_and(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Bool(a), PengLiteral::Bool(b)) => {
            Some(PengLiteral::Bool(*a && *b))
        }

        _ => None,
    }
}

fn fold_or(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Bool(a), PengLiteral::Bool(b)) => {
            Some(PengLiteral::Bool(*a || *b))
        }

        _ => None,
    }
}

fn fold_equals(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Nil, PengLiteral::Nil) => Some(PengLiteral::Bool(true)),

        (PengLiteral::Int(a), PengLiteral::Int(b)) => Some(PengLiteral::Bool(*a == *b)),
        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => Some(PengLiteral::Bool(*a == *b)),
        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => Some(PengLiteral::Bool(*a == *b)),

        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => Some(PengLiteral::Bool(*a == *b)),
        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => Some(PengLiteral::Bool(*a == *b)),

        (PengLiteral::Bool(a), PengLiteral::Bool(b)) => Some(PengLiteral::Bool(*a == *b)),

        (PengLiteral::String(a), PengLiteral::String(b)) => Some(PengLiteral::Bool(a == b)),

        _ => None,
    }
}

fn fold_not_equals(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match fold_equals(left, right) {
        Some(PengLiteral::Bool(v)) => Some(PengLiteral::Bool(!v)),
        _ => None,
    }
}

fn fold_greater_than(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => Some(PengLiteral::Bool(*a > *b)),
        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => Some(PengLiteral::Bool(*a > *b)),
        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => Some(PengLiteral::Bool(*a > *b)),
        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => Some(PengLiteral::Bool(*a > *b)),
        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => Some(PengLiteral::Bool(*a > *b)),
        _ => None,
    }
}

fn fold_greater_equals_than(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => Some(PengLiteral::Bool(*a >= *b)),
        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => Some(PengLiteral::Bool(*a >= *b)),
        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => Some(PengLiteral::Bool(*a >= *b)),
        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => Some(PengLiteral::Bool(*a >= *b)),
        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => Some(PengLiteral::Bool(*a >= *b)),
        _ => None,
    }
}

fn fold_less_than(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => Some(PengLiteral::Bool(*a < *b)),
        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => Some(PengLiteral::Bool(*a < *b)),
        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => Some(PengLiteral::Bool(*a < *b)),
        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => Some(PengLiteral::Bool(*a < *b)),
        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => Some(PengLiteral::Bool(*a < *b)),
        _ => None,
    }
}

fn fold_less_equals_than(left: &PengLiteral, right: &PengLiteral) -> Option<PengLiteral> {
    match (left, right) {
        (PengLiteral::Int(a), PengLiteral::Int(b)) => Some(PengLiteral::Bool(*a <= *b)),
        (PengLiteral::Uint(a), PengLiteral::Uint(b)) => Some(PengLiteral::Bool(*a <= *b)),
        (PengLiteral::Byte(a), PengLiteral::Byte(b)) => Some(PengLiteral::Bool(*a <= *b)),
        (PengLiteral::Float32(a), PengLiteral::Float32(b)) => Some(PengLiteral::Bool(*a <= *b)),
        (PengLiteral::Float64(a), PengLiteral::Float64(b)) => Some(PengLiteral::Bool(*a <= *b)),
        _ => None,
    }
}