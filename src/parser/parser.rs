use std::iter::Peekable;
use std::slice::Iter;
use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub type PengPeekablePositionedToken<'a> = Peekable<Iter<'a, PengPositionedToken>>;

pub fn parse(ptokens: Vec<PengPositionedToken>) -> Result<PengAST, PengError> {
    let first_pos = {
        if let Some(aux) = ptokens.get(0) {
            aux.position.clone()
        } else {
            return Ok(
                PengAST{
                    program: Vec::new(),
                },
            );
        }
    };
    let mut ptokens_iter: PengPeekablePositionedToken = ptokens.iter().peekable();
    match parse_program(
        &mut ptokens_iter,
    ) {
        Ok(root) => return Ok( root ),
        Err(e) => Err(e),
    }
}