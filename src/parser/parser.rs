use std::iter::Peekable;
use std::slice::Iter;
use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub type PengPeekablePositionedToken<'a> = Peekable<Iter<'a, PengPositionedToken>>;

pub fn parse(ptokens: Vec<PengPositionedToken>) -> Result<PengAST, PengError> {
    if ptokens.is_empty() {
        return Ok(
            PengAST {
                program: Vec::new(),
            }
        );
    }

    let mut ptokens_iter: PengPeekablePositionedToken = ptokens.iter().peekable();
    match parse_program(
        &mut ptokens_iter,
    ) {
        Ok(root) => return Ok( root ),
        Err(e) => Err(e),
    }
}
