use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_type_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let type_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected type literal".to_string()));
        }
    };

    match &type_token.value {
        PengToken::Type => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'type'".to_string(),
                type_token.position.clone(),
            ));
        }
    }

    let supers = match parse_type_supers(ptokens) {
        Ok(supers) => supers,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_literal".to_string(),
            )));
        }
    };

    let (fields, functions) = match parse_type_members(ptokens) {
        Ok(members) => members,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_literal".to_string(),
            )));
        }
    };

    literal_expr(
        PengLiteral::Type(PengTypeLiteral {
            supers,
            fields,
            functions,
        }),
        type_token.position.clone(),
    )
}
