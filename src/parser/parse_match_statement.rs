use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_match_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let match_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected match statement".to_string(),
            ));
        }
    };

    match &match_token.value {
        PengToken::Match => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'match'".to_string(),
                match_token.position.clone(),
            ));
        }
    }

    let value = match parse_expression(ptokens) {
        Ok(value) => value,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_match_statement".to_string(),
            )));
        }
    };

    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected '{' after match value".to_string(),
                value.position.clone(),
            ));
        }
    };

    match &open_token.value {
        PengToken::LeftCurlyBrace => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '{' after match value".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let mut arms = Vec::new();
    let mut elsing = None;
    let mut found_else = false;

    loop {
        let token = match ptokens.peek() {
            Some(token) => (*token).clone(),
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

            PengToken::Else => {
                ptokens.next();

                let body_statement = match parse_block_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_match_statement".to_string(),
                        )));
                    }
                };

                let body = match block_statements(body_statement, "expected else body".to_string())
                {
                    Ok(body) => body,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_match_statement".to_string(),
                        )));
                    }
                };

                elsing = Some(body);
                found_else = true;
            }

            _ => {
                if found_else {
                    return Err(PengError::new_positioned_message(
                        "else must be the last match arm".to_string(),
                        token.position.clone(),
                    ));
                }

                let pattern = match parse_expression(ptokens) {
                    Ok(pattern) => pattern,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_match_statement".to_string(),
                        )));
                    }
                };

                let body_statement = match parse_block_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_match_statement".to_string(),
                        )));
                    }
                };

                let body =
                    match block_statements(body_statement, "expected match arm body".to_string()) {
                        Ok(body) => body,
                        Err(e) => {
                            return Err(e.push(PengError::SyntaxError(
                                "failed while parsing parse_match_statement".to_string(),
                            )));
                        }
                    };

                arms.push(PengMatchArm { pattern, body });
            }
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Match(PengMatchStatement {
            value,
            arms,
            elsing,
        }),
        position: match_token.position.clone(),
    })
}
