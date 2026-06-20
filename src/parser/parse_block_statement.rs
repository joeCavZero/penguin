use crate::core::*;
use crate::parser::*;

pub fn parse_block_statement(
    _ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    Err(PengError::new_message(
        "parse_block_statement not implemented".to_string()
    ))
}
