use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parse_utils::block_statements;

pub fn parse_loop_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let loop_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected loop statement".to_string(),
            ));
        }
    };

    match &loop_token.value {
        PengToken::Loop => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'loop'".to_string(),
                loop_token.position.clone(),
            ));
        }
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => return Err(e),
    };

    let body = match block_statements(
        body_statement,
        "expected loop body".to_string(),
    ) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    Ok(PengPositioned {
        value: PengStatement::Loop(body),
        position: loop_token.position.clone(),
    })
}
