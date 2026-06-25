use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_vector_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected vector literal".to_string(),
            ));
        }
    };

    match &open_token.value {
        PengToken::LeftBracket => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '['".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let mut values = Vec::new();

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ']'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &token.value {
            PengToken::RightBracket => {
                ptokens.next();
                break;
            }
            _ => {}
        }

        let value = match parse_expression(ptokens) {
            Ok(expression) => expression,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_vector_literal".to_string(),
                )));
            }
        };

        values.push(value);

        let separator = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ']'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &separator.value {
            PengToken::Comma => {}
            PengToken::RightBracket => break,
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ']'".to_string(),
                    separator.position.clone(),
                ));
            }
        }
    }

    literal_expr(PengLiteral::Vector(values), open_token.position.clone())
}
