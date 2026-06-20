use crate::core::*;
use crate::parser::*;

pub fn parse_type_declaration_statement(
    _ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    Err(PengError::new_message(
        "parse_type_declaration_statement not implemented".to_string()
    ))
}
