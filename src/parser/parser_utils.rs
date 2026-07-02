use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;

pub fn consume_optional_semicolon(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<(), PengError> {
    loop {
        let has_semicolon = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::Semicolon => true,
                _ => false,
            },
            None => false,
        };

        if !has_semicolon {
            break;
        }

        match ptokens.next() {
            Some(_) => {}
            None => {
                return Err(PengError::SyntaxError("expected semicolon".to_string()));
            }
        }
    }

    Ok(())
}

pub fn expect_identifier(
    ptokens: &mut PengPeekablePositionedToken,
    message: String,
    fallback_position: PengPosition,
) -> Result<PengPositioned<String>, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                message,
                fallback_position,
            ));
        }
    };

    match &token.value {
        PengToken::Identifier(name) => Ok(PengPositioned {
            value: name.clone(),
            position: token.position.clone(),
        }),
        _ => Err(PengError::new_positioned_message(
            message,
            token.position.clone(),
        )),
    }
}

pub fn block_statements(
    statement: PengPositionedStatement,
    message: String,
) -> Result<Vec<PengPositionedStatement>, PengError> {
    match statement.value {
        PengStatement::Block(statements) => Ok(statements),
        _ => Err(PengError::new_positioned_message(
            message,
            statement.position,
        )),
    }
}
