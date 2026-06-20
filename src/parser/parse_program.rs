use crate::core::*;
use crate::parser::*;

pub fn parse_program(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengAST, PengError> {
    let mut program = Vec::new();

    while ptokens.peek().is_some() {
        match parse_statement(ptokens) {
            Ok(stmt) => {
                program.push(stmt);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(PengAST { program })
}
