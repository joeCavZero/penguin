use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parse_statement::*;

pub fn parse_script(ptokens: Vec<PengPositionedToken>) -> Result<PengAST, PengError> {
    let mut ptokens_iter: PengPeekablePositionedToken = ptokens.iter().peekable();
    let mut statements = Vec::new();

    while ptokens_iter.peek().is_some() {
        let statement = match parse_statement(&mut ptokens_iter) {
            Ok(statement) => statement,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_script".to_string(),
                )));
            }
        };

        statements.push(statement);
    }

    Ok(PengAST::Script(statements))
}
