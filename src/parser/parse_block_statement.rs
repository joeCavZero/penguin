use crate::core::*;
use crate::lexer::*;
use crate::parser::parse_statement::*;
use crate::parser::parser::*;

pub fn parse_block_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected block statement".to_string(),
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

    let mut statements = Vec::new();

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
            _ => {
                let statement = match parse_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_block_statement".to_string(),
                        )));
                    }
                };

                statements.push(statement);
            }
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Block(statements),
        position: open_token.position.clone(),
    })
}
