use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parse_block_statement::*;
use crate::parser::parse_function_params::*;
use crate::parser::parse_type_expression::*;

pub fn parse_function_literal(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let func_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected function literal".to_string(),
            ));
        }
    };

    match &func_token.value {
        PengToken::Func => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'func'".to_string(),
                func_token.position.clone(),
            ));
        }
    }

    let params = match parse_function_params_declaration(ptokens) {
        Ok(params) => params,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_function_literal".to_string(),
            )));
        }
    };

    let has_return_type = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Arrow => true,
            _ => false,
        },
        None => false,
    };

    let return_type = if has_return_type {
        ptokens.next();

        match parse_type_expression(ptokens) {
            Ok(type_expression) => Some(type_expression),
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_function_literal".to_string(),
                )));
            }
        }
    } else {
        None
    };

    match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::LeftCurlyBrace => {}
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected function body".to_string(),
                    token.position.clone(),
                ));
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected function body".to_string(),
                func_token.position.clone(),
            ));
        }
    }

    let body_statement = match parse_block_statement(ptokens) {
        Ok(statement) => statement,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_function_literal".to_string(),
            )));
        }
    };

    let body = match body_statement.value {
        PengStatement::Block(statements) => statements,
        _ => {
            return Err(PengError::new_positioned_message(
                "expected function body".to_string(),
                body_statement.position.clone(),
            ));
        }
    };

    let literal = PengPositioned {
        value: PengLiteral::Function(PengFunctionLiteral {
            params,
            return_type,
            body,
        }),
        position: func_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengExpression::Literal(literal),
        position: func_token.position.clone(),
    })
}
