use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_function_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let func_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
                "expected function declaration".to_string(),
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

    let name = match parse_function_name(ptokens, &func_token.position) {
        Ok(name) => name,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_function_declaration_statement".to_string(),
            )));
        }
    };

    let params = match parse_function_params_declaration(ptokens) {
        Ok(params) => params,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_function_declaration_statement".to_string(),
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
                    "failed while parsing parse_function_declaration_statement".to_string(),
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
                "failed while parsing parse_function_declaration_statement".to_string(),
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

    let declaration = PengPositioned {
        value: PengFunctionDeclaration {
            name,
            params,
            return_type,
            body,
        },
        position: func_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Function(
            declaration,
        ))),
        position: func_token.position.clone(),
    })
}

fn parse_function_name(
    ptokens: &mut PengPeekablePositionedToken,
    func_position: &PengPosition,
) -> Result<PengPositioned<String>, PengError> {
    let token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_positioned_message(
                "expected function name".to_string(),
                func_position.clone(),
            ));
        }
    };

    match &token.value {
        PengToken::Identifier(name) => Ok(PengPositioned {
            value: name.clone(),
            position: token.position.clone(),
        }),
        _ => Err(PengError::new_positioned_message(
            "expected function name".to_string(),
            token.position.clone(),
        )),
    }
}
