use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parse_utils::consume_optional_semicolon;

pub fn parse_continue_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected 'continue'".to_string(),
            ));
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
        Err(e) => return Err(e),
    }

    Ok(PengPositioned {
        value: PengStatement::Continue,
        position: token.position.clone(),
    })
}
