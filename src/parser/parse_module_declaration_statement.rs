use crate::core::*;
use crate::lexer::*;
use crate::parser::parse_declaration_statement::*;
use crate::parser::parser::*;
use crate::parser::parser_utils::expect_identifier;

pub fn parse_module_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let mod_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected module declaration".to_string(),
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

    let name = match expect_identifier(
        ptokens,
        "expected module name".to_string(),
        mod_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_module_declaration_statement".to_string(),
            )));
        }
    };

    let body = match parse_declaration_body(ptokens) {
        Ok(body) => body,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_module_declaration_statement".to_string(),
            )));
        }
    };

    let declaration = PengPositioned {
        value: PengModuleDeclaration { name, body },
        position: mod_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Module(
            declaration,
        ))),
        position: mod_token.position.clone(),
    })
}
