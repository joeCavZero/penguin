use crate::core::*;
use crate::parser::*;

pub fn parse_if_statement(
    _ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    Err(PengError::new_message(
        "parse_if_statement not implemented".to_string()
    ))
}
