use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {

    let token = match ptokens.peek() {
        Some(t) => t,
        None => {
            return Err(
                PengError::new_message(
                    "expected statement".to_string()
                )
            );
        }
    };

    let statement = match &token.value {
        PengToken::LeftCurlyBrace => {
            parse_block_statement(ptokens)
        }

        PengToken::Const
        | PengToken::Var
        | PengToken::Func
        | PengToken::Type
        | PengToken::Union
        | PengToken::Mod
        | PengToken::Oper => {
            parse_declaration_statement(ptokens)
        }

        PengToken::Return => {
            parse_return_statement(ptokens)
        }

        PengToken::If => {
            parse_if_statement(ptokens)
        }

        PengToken::Match => {
            parse_match_statement(ptokens)
        }

        PengToken::While => {
            parse_while_statement(ptokens)
        }

        PengToken::For => {
            parse_for_statement(ptokens)
        }

        PengToken::Loop => {
            parse_loop_statement(ptokens)
        }

        PengToken::Break => {
            parse_break_statement(ptokens)
        }

        PengToken::Continue => {
            parse_continue_statement(ptokens)
        }

        _ => {
            parse_expression_statement(ptokens)
        }
    };

    let statement = match statement {
        Ok(statement) => statement,
        Err(e) => return Err(e),
    };

    match consume_optional_semicolon(ptokens) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(statement)
}
