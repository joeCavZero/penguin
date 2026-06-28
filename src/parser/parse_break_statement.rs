use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::consume_optional_semicolon;
use crate::parser::parser::*;

pub fn parse_break_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected 'break'".to_string()));
        }
    };

    match &token.value {
        PengToken::Break => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'break'".to_string(),
                token.position.clone(),
            ));
        }
    }

    match consume_optional_semicolon(ptokens) {
        Ok(_) => {}
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_break_statement".to_string(),
            )));
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Break,
        position: token.position.clone(),
    })
}
