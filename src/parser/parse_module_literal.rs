use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_module_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let mod_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected module literal".to_string(),
            ));
        }
    };

    match &mod_token.value {
        PengToken::Mod => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'mod'".to_string(),
                mod_token.position.clone(),
            ));
        }
    }

    let body = match parse_declaration_body(ptokens) {
        Ok(body) => body,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_module_literal".to_string(),
            )));
        }
    };

    literal_expr(
        PengLiteral::Module(PengModuleLiteral { body }),
        mod_token.position.clone(),
    )
}
