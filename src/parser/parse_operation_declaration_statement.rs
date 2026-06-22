use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parser_utils::{
    block_statements,
    expect_identifier,
};

pub fn parse_operation_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let oper_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected operation declaration".to_string(),
            ));
        }
    };

    match &oper_token.value {
        PengToken::Oper => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'oper'".to_string(),
                oper_token.position.clone(),
            ));
        }
    }

    let name = match expect_identifier(
        ptokens,
        "expected operation name".to_string(),
        oper_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let params = match parse_function_params_declaration(ptokens) {
        Ok(params) => params,
        Err(e) => return Err(e),
    };

    if params.len() != 2 {
        return Err(PengError::new_positioned_message(
            "operation declarations require exactly two parameters".to_string(),
            name.position.clone(),
        ));
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => return Err(e),
    };

    let body = match block_statements(
        body_statement,
        "expected operation body".to_string(),
    ) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    let declaration = PengPositioned {
        value: PengOperationDeclaration {
            name,
            params,
            body,
        },
        position: oper_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(
            PengBinded::Mutable(
                PengDeclaration::Operation(declaration)
            )
        ),
        position: oper_token.position.clone(),
    })
}
