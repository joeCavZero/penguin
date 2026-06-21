use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parse_utils::expect_identifier;

pub fn parse_module_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let mod_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected module declaration".to_string(),
            ));
        }
    };

    match &mod_token.value {
        PengToken::Mod => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'mod'".to_string(),
                mod_token.position.clone(),
            ));
        }
    }

    let name = match expect_identifier(
        ptokens,
        "expected module name".to_string(),
        mod_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let body = match parse_declaration_body(ptokens) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    let declaration = PengPositioned {
        value: PengModuleDeclaration {
            name,
            body,
        },
        position: mod_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(
            PengDeclaration::Module(declaration)
        ),
        position: mod_token.position.clone(),
    })
}

pub fn parse_declaration_body(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengDeclaration>, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected declaration body".to_string(),
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

    let mut declarations = Vec::new();

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected '}'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &token.value {
            PengToken::RightCurlyBrace => {
                ptokens.next();
                break;
            }
            PengToken::Var
            | PengToken::Func
            | PengToken::Type
            | PengToken::Union
            | PengToken::Mod
            | PengToken::Oper => {
                let statement = match parse_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => return Err(e),
                };

                match statement.value {
                    PengStatement::Declaration(declaration) => {
                        declarations.push(declaration);
                    }
                    _ => {
                        return Err(PengError::new_positioned_message(
                            "module body only accepts declarations".to_string(),
                            statement.position,
                        ));
                    }
                }
            }
            _ => {
                return Err(PengError::new_positioned_message(
                    "module body only accepts declarations".to_string(),
                    token.position.clone(),
                ));
            }
        }
    }

    Ok(declarations)
}
