use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::{block_statements, expect_identifier};
use crate::parser::*;

pub fn parse_operation_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let oper_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
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
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_declaration_statement".to_string(),
            )));
        }
    };

    let params = match parse_function_params_declaration(ptokens) {
        Ok(params) => params,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_declaration_statement".to_string(),
            )));
        }
    };

    if params.len() != 2 {
        return Err(PengError::new_positioned_message(
            "operation declarations require exactly two parameters".to_string(),
            name.position.clone(),
        ));
    }

    let return_type = match parse_optional_arrow_return_type(
        ptokens,
        "failed while parsing parse_operation_declaration_statement",
    ) {
        Ok(return_type) => return_type,
        Err(e) => return Err(e),
    };

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_declaration_statement".to_string(),
            )));
        }
    };

    let body = match block_statements(body_statement, "expected operation body".to_string()) {
        Ok(body) => body,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_declaration_statement".to_string(),
            )));
        }
    };

    let declaration = PengPositioned {
        value: PengOperationDeclaration {
            name,
            params,
            return_type,
            body,
        },
        position: oper_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Operation(
            declaration,
        ))),
        position: oper_token.position.clone(),
    })
}

pub fn parse_optional_arrow_return_type(
    ptokens: &mut PengPeekablePositionedToken,
    error_context: &str,
) -> Result<Option<PengPositioned<PengTypeExpression>>, PengError> {
    let has_return_type = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Arrow => true,
            _ => false,
        },
        None => false,
    };

    if !has_return_type {
        return Ok(None);
    }

    ptokens.next();

    match parse_type_expression(ptokens) {
        Ok(type_expression) => Ok(Some(type_expression)),
        Err(e) => Err(e.push(PengError::SyntaxError(
            error_context.to_string(),
        ))),
    }
}