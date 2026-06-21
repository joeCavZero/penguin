use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parse_utils::consume_optional_semicolon;

pub fn parse_expression_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    parse_expression_or_assignment_statement(ptokens, true)
}

pub fn parse_expression_or_assignment_statement(
    ptokens: &mut PengPeekablePositionedToken,
    consume_semicolon: bool,
) -> Result<PengPositionedStatement, PengError> {
    let expression = match parse_expression(ptokens) {
        Ok(expression) => expression,
        Err(e) => return Err(e),
    };

    let position = expression.position.clone();
    let assignment_token = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Equals
            | PengToken::PlusEquals
            | PengToken::MinusEquals
            | PengToken::AsteriskEquals
            | PengToken::SlashEquals
            | PengToken::DoubleAsteriskEquals
            | PengToken::PercentEquals => Some((*token).clone()),
            _ => None,
        },
        None => None,
    };

    let value = if assignment_token.is_some() {
        ptokens.next();
        Some(match parse_expression(ptokens) {
            Ok(value) => value,
            Err(e) => return Err(e),
        })
    } else {
        None
    };

    if consume_semicolon {
        match consume_optional_semicolon(ptokens) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    let statement = match (assignment_token, value) {
        (Some(token), Some(value)) => {
            if !is_assignable(&expression.value) {
                return Err(PengError::new_positioned_message(
                    "invalid assignment target".to_string(),
                    expression.position.clone(),
                ));
            }

            let assignment = match token.value {
                PengToken::Equals => PengAssignStatement::Assign {
                    target: expression,
                    value,
                },
                PengToken::PlusEquals => PengAssignStatement::AddAssign {
                    target: expression,
                    value,
                },
                PengToken::MinusEquals => PengAssignStatement::SubtractAssign {
                    target: expression,
                    value,
                },
                PengToken::AsteriskEquals => PengAssignStatement::MultiplyAssign {
                    target: expression,
                    value,
                },
                PengToken::SlashEquals => PengAssignStatement::DivideAssign {
                    target: expression,
                    value,
                },
                PengToken::DoubleAsteriskEquals => PengAssignStatement::PowerAssign {
                    target: expression,
                    value,
                },
                PengToken::PercentEquals => PengAssignStatement::RemainderAssign {
                    target: expression,
                    value,
                },
                _ => {
                    return Err(PengError::new_positioned_message(
                        "expected assignment operator".to_string(),
                        token.position.clone(),
                    ));
                }
            };

            PengStatement::Assign(assignment)
        }
        _ => PengStatement::Expression(expression),
    };

    Ok(PengPositioned {
        value: statement,
        position,
    })
}

fn is_assignable(expression: &PengExpression) -> bool {
    matches!(
        expression,
        PengExpression::Identifier(_) | PengExpression::AttributeAccess(_) | PengExpression::Index(_)
    )
}
