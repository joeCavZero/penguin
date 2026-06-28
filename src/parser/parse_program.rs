use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parse_statement::*;

pub fn parse_program(ptokens: Vec<PengPositionedToken>) -> Result<PengAST, PengError> {
    let mut ptokens_iter: PengPeekablePositionedToken = ptokens.iter().peekable();
    let mut declarations = Vec::new();

    while ptokens_iter.peek().is_some() {
        let statement = match parse_statement(&mut ptokens_iter) {
            Ok(stmt) => stmt,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_program".to_string(),
                )));
            }
        };

        match statement.value {
            PengStatement::Declaration(declaration) => {
                declarations.push(declaration);
            }
            _ => {
                return Err(PengError::new_positioned_message(
                    "program mode only accepts declarations".to_string(),
                    statement.position.clone(),
                ));
            }
        }
    }

    Ok(PengAST::Program(declarations))
}
