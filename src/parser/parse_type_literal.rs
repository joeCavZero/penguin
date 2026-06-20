use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_type_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let type_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected type literal".to_string(),
            ));
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

    let generics = match parse_function_generics(ptokens) {
        Ok(generics) => generics,
        Err(e) => return Err(e),
    };

    let supers = match parse_type_supers(ptokens) {
        Ok(supers) => supers,
        Err(e) => return Err(e),
    };

    let (fields, functions) = match parse_type_members(ptokens) {
        Ok(members) => members,
        Err(e) => return Err(e),
    };

    literal_expr(
        PengLiteral::Type(PengTypeLiteral {
            generics,
            supers,
            fields,
            functions,
        }),
        type_token.position.clone(),
    )
}
