use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parse_block_statement::*;
use crate::parser::parse_expression::*;

pub fn parse_while_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let while_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected while statement".to_string(),
            ));
        }
    };

    match &while_token.value {
        PengToken::While => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'while'".to_string(),
                while_token.position.clone(),
            ));
        }
    }

    let condition = match parse_expression(ptokens) {
        Ok(expression) => expression,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_while_statement".to_string(),
            )));
        }
    };

    match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftCurlyBrace => {}
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected block after while condition".to_string(),
                    token.position.clone(),
                ));
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected block after while condition".to_string(),
                condition.position.clone(),
            ));
        }
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_while_statement".to_string(),
            )));
        }
    };

    let body = match body_statement.value {
        PengStatement::Block(statements) => statements,
        _ => {
            return Err(PengError::new_positioned_message(
                "expected block after while condition".to_string(),
                body_statement.position.clone(),
            ));
        }
    };

    Ok(PengPositioned {
        value: PengStatement::While(PengWhileStatement { condition, body }),
        position: while_token.position.clone(),
    })
}
