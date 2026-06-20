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

    match &token.value {
        PengToken::LeftCurlyBrace => {
            parse_block_statement(ptokens)
        }

        PengToken::Var => {
            parse_variable_declaration_statement(ptokens)
        }

        PengToken::Func => {
            parse_function_declaration_statement(ptokens)
        }

        PengToken::Type => {
            parse_type_declaration_statement(ptokens)
        }

        PengToken::Mod => {
            parse_module_declaration_statement(ptokens)
        }

        PengToken::Return => {
            parse_return_statement(ptokens)
        }

        PengToken::If => {
            parse_if_statement(ptokens)
        }

        PengToken::While => {
            parse_while_statement(ptokens)
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
    }
}