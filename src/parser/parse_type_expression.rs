use crate::core::*;
use crate::parser::*;

pub fn parse_type_expression(
    _ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedTypeExpression, PengError> {
    Err(PengError::new_message(
        "parse_type_expression not implemented".to_string(),
    ))
}