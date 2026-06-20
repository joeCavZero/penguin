use crate::core::*;
use crate::parser::*;

pub fn parse_function_declaration_statement(
    _ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    Err(PengError::new_message(
        "parse_function_declaration_statement not implemented".to_string()
    ))
}
