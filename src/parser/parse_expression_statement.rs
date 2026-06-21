use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parser_utils::consume_optional_semicolon;

pub fn parse_expression_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    parse_expression_or_assignment_statement(ptokens, true)
}

pub fn parse_expression_or_assignment_statement(
    ptokens: &mut PengPeekablePositionedToken,
    consume_semicolon: bool,
) -> Result<PengPositionedStatement, PengError> {
    let expression_start = ptokens.clone();
    let declaration_value = match parse_expression_before_as(ptokens) {
        Ok(expression) => expression,
        Err(e) => return Err(e),
    };

    let mut declaration_lookahead = ptokens.clone();
    let as_declaration = match declaration_lookahead.next() {
        Some(as_token) => match &as_token.value {
            PengToken::As => match declaration_lookahead.next() {
                Some(name_token) => match &name_token.value {
                    PengToken::Identifier(_) => {
                        Some(as_token.clone())
                    }
                    _ => None,
                },
                None => None,
            },
            _ => None,
        },
        None => None,
    };

    match as_declaration {
        Some(as_token) => {
            match ptokens.next() {
                Some(_) => {}
                None => {
                    return Err(PengError::new_positioned_message(
                        "expected 'as' declaration".to_string(),
                        as_token.position.clone(),
                    ));
                }
            }

            let name = match crate::parser::parser_utils::expect_identifier(
                ptokens,
                "expected declaration name after 'as'".to_string(),
                as_token.position.clone(),
            ) {
                Ok(name) => name,
                Err(e) => return Err(e),
            };

            if consume_semicolon {
                match consume_optional_semicolon(ptokens) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            let position = declaration_value.position.clone();
            let declaration = PengPositioned {
                value: PengAsDeclaration {
                    name,
                    value: declaration_value,
                },
                position: as_token.position.clone(),
            };

            return Ok(PengPositioned {
                value: PengStatement::Declaration(
                    PengDeclaration::As(declaration),
                ),
                position,
            });
        }
        None => {}
    }

    *ptokens = expression_start;
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
