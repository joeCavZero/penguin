use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::expect_identifier;
use crate::parser::*;

pub fn parse_union_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let union_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected union declaration".to_string(),
            ));
        }
    };

    match &union_token.value {
        PengToken::Union => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'union'".to_string(),
                union_token.position.clone(),
            ));
        }
    }

    let name = match expect_identifier(
        ptokens,
        "expected union name".to_string(),
        union_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_union_declaration_statement".to_string(),
            )));
        }
    };

    let equals_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected '=' after union name".to_string(),
                name.position.clone(),
            ));
        }
    };

    match &equals_token.value {
        PengToken::Equals => {}
        _ => {
            return Err(PengError::ExpectedToken {
                expected: "=".to_string(),
                found: format!("{:?}", equals_token.value),
            });
        }
    }

    let value = match parse_type_expression(ptokens) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_union_declaration_statement".to_string(),
            )));
        }
    };

    let declaration = PengPositioned {
        value: PengTypeDeclaration {
            name,
            value: Some(value),
            supers: Vec::new(),
            fields: Vec::new(),
            functions: Vec::new(),
        },
        position: union_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Type(declaration))),
        position: union_token.position.clone(),
    })
}

pub fn parse_union_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let union_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected union expression".to_string(),
            ));
        }
    };

    match &union_token.value {
        PengToken::Union => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'union'".to_string(),
                union_token.position.clone(),
            ));
        }
    }

    let value = match parse_type_expression(ptokens) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_union_declaration_statement".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        value: PengExpression::Type(value),
        position: union_token.position.clone(),
    })
}
