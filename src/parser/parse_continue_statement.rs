use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parser_utils::consume_optional_semicolon;

pub fn parse_continue_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected 'continue'".to_string()));
        }
    };

    match &token.value {
        PengToken::Continue => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'continue'".to_string(),
                token.position.clone(),
            ));
        }
    }

    match consume_optional_semicolon(ptokens) {
        Ok(_) => {}
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_continue_statement".to_string(),
            )));
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Continue,
        position: token.position.clone(),
    })
}
