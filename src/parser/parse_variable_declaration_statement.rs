use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parser_utils::consume_optional_semicolon;

pub fn parse_variable_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let var_token = match ptokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::new_message(
                "expected variable declaration".to_string(),
            ));
        }
    };

    match &var_token.value {
        PengToken::Var => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'var'".to_string(),
                var_token.position.clone(),
            ));
        }
    }

    let name = match ptokens.next() {
        Some(t) => {
            match &t.value {
                PengToken::Identifier(name) => {
                    PengPositioned {
                        value: name.clone(),
                        position: t.position.clone(),
                    }
                }
                _ => {
                    return Err(PengError::new_positioned_message(
                        "expected variable name".to_string(),
                        t.position.clone(),
                    ));
                }
            }
        }
        None => {
            return Err(PengError::new_positioned_message(
                "expected variable name".to_string(),
                var_token.position.clone(),
            ));
        }
    };

    let mut type_hint = None;

    let has_colon = match ptokens.peek() {
        Some(t) => {
            match &t.value {
                PengToken::Colon => true,
                _ => false,
            }
        }
        None => false,
    };

    if has_colon {
        ptokens.next();

        match parse_type_expression(ptokens) {
            Ok(t) => {
                type_hint = Some(t);
            }
            Err(e) => return Err(e),
        }
    }

    let mut value = None;

    let has_equals = match ptokens.peek() {
        Some(t) => {
            match &t.value {
                PengToken::Equals => true,
                _ => false,
            }
        }
        None => false,
    };

    if has_equals {
        ptokens.next();

        match parse_expression(ptokens) {
            Ok(expr) => {
                value = Some(expr);
            }
            Err(e) => return Err(e),
        }
    }

    match consume_optional_semicolon(ptokens) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let declaration = PengVariableDeclaration {
        name,
        type_hint,
        value,
    };

    let positioned_declaration = PengPositioned {
        value: declaration,
        position: var_token.position.clone(),
    };

    let stmt = PengStatement::Declaration(
        PengDeclaration::Variable(positioned_declaration)
    );

    Ok(PengPositioned {
        value: stmt,
        position: var_token.position.clone(),
    })
}
