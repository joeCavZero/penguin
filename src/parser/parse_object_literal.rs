use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::expect_identifier;
use crate::parser::*;

pub fn parse_object_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected object literal".to_string(),
            ));
        }
    };

    match &open_token.value {
        PengToken::LeftCurlyBrace => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '{'".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let fields = match parse_object_fields(ptokens, open_token.position.clone()) {
        Ok(fields) => fields,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_object_literal".to_string(),
            )));
        }
    };

    literal_expr(PengLiteral::Object(fields), open_token.position.clone())
}

pub fn parse_object_fields(
    ptokens: &mut PengPeekablePositionedToken,
    open_position: PengPosition,
) -> Result<Vec<PengObjectFieldLiteral>, PengError> {
    let mut fields = Vec::new();

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected '}'".to_string(),
                    open_position,
                ));
            }
        };

        match &token.value {
            PengToken::RightCurlyBrace => {
                ptokens.next();
                break;
            }
            _ => {}
        }

        let name = match expect_identifier(
            ptokens,
            "expected object field name".to_string(),
            open_position.clone(),
        ) {
            Ok(name) => name,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_object_literal".to_string(),
                )));
            }
        };

        let equals_token = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected '='".to_string(),
                    name.position.clone(),
                ));
            }
        };

        match &equals_token.value {
            PengToken::Equals => {}
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected '='".to_string(),
                    equals_token.position.clone(),
                ));
            }
        }

        let value = match parse_expression(ptokens) {
            Ok(expression) => expression,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_object_literal".to_string(),
                )));
            }
        };

        fields.push(PengObjectFieldLiteral { name, value });

        let separator = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or '}'".to_string(),
                    open_position,
                ));
            }
        };

        match &separator.value {
            PengToken::Comma => {}
            PengToken::RightCurlyBrace => break,
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or '}'".to_string(),
                    separator.position.clone(),
                ));
            }
        }
    }

    Ok(fields)
}
