use crate::core::*;
use crate::lexer::*;
use crate::parser::parse_block_statement::*;
use crate::parser::parse_expression::*;
use crate::parser::parser::*;

pub fn parse_if_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let if_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected if statement".to_string()));
        }
    };

    match &if_token.value {
        PengToken::If => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'if'".to_string(),
                if_token.position.clone(),
            ));
        }
    }

    let condition = match parse_expression(ptokens) {
        Ok(expression) => expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_if_statement".to_string(),
            )));
        }
    };

    match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftCurlyBrace => {}
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected block after if condition".to_string(),
                    token.position.clone(),
                ));
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected block after if condition".to_string(),
                condition.position.clone(),
            ));
        }
    }

    let then_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_if_statement".to_string(),
            )));
        }
    };

    let then_branch = match then_statement.value {
        PengStatement::Block(statements) => statements,
        _ => {
            return Err(PengError::new_positioned_message(
                "expected block after if condition".to_string(),
                then_statement.position.clone(),
            ));
        }
    };

    let has_else = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Else => true,
            _ => false,
        },
        None => false,
    };

    let else_branch = if has_else {
        let else_token = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected 'else'".to_string(),
                    if_token.position.clone(),
                ));
            }
        };

        match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::LeftCurlyBrace => {}
                _ => {
                    return Err(PengError::new_positioned_message(
                        "expected block after 'else'".to_string(),
                        token.position.clone(),
                    ));
                }
            },
            None => {
                return Err(PengError::new_positioned_message(
                    "expected block after 'else'".to_string(),
                    else_token.position.clone(),
                ));
            }
        }

        let else_statement = match parse_block_statement(ptokens) {
            Ok(statement) => statement,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_if_statement".to_string(),
                )));
            }
        };

        match else_statement.value {
            PengStatement::Block(statements) => Some(statements),
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected block after 'else'".to_string(),
                    else_token.position.clone(),
                ));
            }
        }
    } else {
        None
    };

    Ok(PengPositioned {
        value: PengStatement::If(PengIfStatement {
            condition,
            then_branch,
            else_branch,
        }),
        position: if_token.position.clone(),
    })
}
