use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::block_statements;
use crate::parser::*;

pub fn parse_for_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let for_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message("expected for statement".to_string()));
        }
    };

    match &for_token.value {
        PengToken::For => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'for'".to_string(),
                for_token.position.clone(),
            ));
        }
    }

    let is_each = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Each => true,
            _ => false,
        },
        None => false,
    };

    if is_each {
        return parse_for_each_statement(ptokens, for_token);
    }

    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected '('".to_string(),
                for_token.position.clone(),
            ));
        }
    };

    match &open_token.value {
        PengToken::LeftParenthesis => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '('".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let initializer = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Semicolon => {
                ptokens.next();
                None
            }
            PengToken::Var => {
                let statement = match parse_variable_declaration_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => return Err(e),
                };

                Some(Box::new(statement))
            }
            _ => {
                let statement = match parse_expression_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => return Err(e),
                };

                Some(Box::new(statement))
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected for initializer".to_string(),
                open_token.position.clone(),
            ));
        }
    };

    let condition = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Semicolon => {
                ptokens.next();
                None
            }
            _ => {
                let expression = match parse_expression(ptokens) {
                    Ok(expression) => expression,
                    Err(e) => return Err(e),
                };

                let semicolon = match ptokens.next() {
                    Some(token) => token,
                    None => {
                        return Err(PengError::new_positioned_message(
                            "expected ';'".to_string(),
                            expression.position.clone(),
                        ));
                    }
                };

                match &semicolon.value {
                    PengToken::Semicolon => {}
                    _ => {
                        return Err(PengError::new_positioned_message(
                            "expected ';'".to_string(),
                            semicolon.position.clone(),
                        ));
                    }
                }

                Some(expression)
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected for condition".to_string(),
                open_token.position.clone(),
            ));
        }
    };

    let increment = match ptokens.peek() {
        Some(token) => {
            match &token.value {
                PengToken::RightParenthesis => None,
                _ => {
                    match crate::parser::parse_expression_statement::parse_expression_or_assignment_statement(
                        ptokens,
                        false,
                    ) {
                        Ok(statement) => Some(Box::new(statement)),
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        None => {
            return Err(PengError::new_positioned_message(
                "expected ')'".to_string(),
                open_token.position.clone(),
            ));
        }
    };

    if let Some(pptk) = ptokens.peek() { 
        match pptk.value {
            PengToken::Semicolon => {ptokens.next();},
            _ => {},
        }
    }

    let close_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected ')'".to_string(),
                open_token.position.clone(),
            ));
        }
    };

    match &close_token.value {
        PengToken::RightParenthesis => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected ')'".to_string(),
                close_token.position.clone(),
            ));
        }
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => return Err(e),
    };

    let body = match block_statements(body_statement, "expected for body".to_string()) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    Ok(PengPositioned {
        value: PengStatement::For(PengForStatement {
            initializer,
            condition,
            increment,
            body,
        }),
        position: for_token.position.clone(),
    })
}

fn parse_for_each_statement(
    ptokens: &mut PengPeekablePositionedToken,
    for_token: &PengPositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    ptokens.next();

    let name = match crate::parser::parser_utils::expect_identifier(
        ptokens,
        "expected for each variable".to_string(),
        for_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let in_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected 'in'".to_string(),
                name.position.clone(),
            ));
        }
    };

    match &in_token.value {
        PengToken::In => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'in'".to_string(),
                in_token.position.clone(),
            ));
        }
    }

    let iterable = match parse_expression(ptokens) {
        Ok(expression) => expression,
        Err(e) => return Err(e),
    };

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => return Err(e),
    };

    let body = match block_statements(body_statement, "expected for each body".to_string()) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    Ok(PengPositioned {
        value: PengStatement::ForEach(PengForEachStatement {
            name,
            iterable,
            body,
        }),
        position: for_token.position.clone(),
    })
}
