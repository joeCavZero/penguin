use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::block_statements;
use crate::parser::*;

pub fn parse_operation_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let oper_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected operation literal".to_string(),
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

    let params = match parse_function_params_declaration(ptokens) {
        Ok(params) => params,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_literal".to_string(),
            )));
        }
    };

    if params.len() != 2 {
        return Err(PengError::new_positioned_message(
            "operation literals require exactly two parameters".to_string(),
            oper_token.position.clone(),
        ));
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_literal".to_string(),
            )));
        }
    };

    let body = match block_statements(body_statement, "expected operation body".to_string()) {
        Ok(body) => body,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_operation_literal".to_string(),
            )));
        }
    };

    literal_expr(
        PengLiteral::Operation(PengOperationLiteral { params, body }),
        oper_token.position.clone(),
    )
}
