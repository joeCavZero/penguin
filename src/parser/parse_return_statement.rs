use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::consume_optional_semicolon;
use crate::parser::*;

pub fn parse_return_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let return_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected return statement".to_string(),
            ));
        }
    };

    match &return_token.value {
        PengToken::Return => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'return'".to_string(),
                return_token.position.clone(),
            ));
        }
    }

    let has_expression = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Semicolon | PengToken::RightCurlyBrace => false,
            _ => true,
        },
        None => false,
    };

    let value = if has_expression {
        match parse_expression(ptokens) {
            Ok(expression) => Some(expression),
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_return_statement".to_string(),
                )));
            }
        }
    } else {
        None
    };

    match consume_optional_semicolon(ptokens) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_return_statement".to_string(),
            )));
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Return(value),
        position: return_token.position.clone(),
    })
}
