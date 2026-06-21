use crate::core::*;
use crate::lexer::*;
use crate::parser::*;
use crate::parser::parse_utils::expect_identifier;

pub fn parse_type_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let type_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected type declaration".to_string(),
            ));
        }
    };

    match &type_token.value {
        PengToken::Type => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'type'".to_string(),
                type_token.position.clone(),
            ));
        }
    }

    let name = match expect_identifier(
        ptokens,
        "expected type name".to_string(),
        type_token.position.clone(),
    ) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let generics = match parse_function_generics(ptokens) {
        Ok(generics) => generics,
        Err(e) => return Err(e),
    };

    let supers = match parse_type_supers(ptokens) {
        Ok(supers) => supers,
        Err(e) => return Err(e),
    };

    let (fields, functions) = match parse_type_members(ptokens) {
        Ok(members) => members,
        Err(e) => return Err(e),
    };

    let declaration = PengPositioned {
        value: PengTypeDeclaration {
            name,
            generics,
            value: None,
            supers,
            fields,
            functions,
        },
        position: type_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(
            PengDeclaration::Type(declaration)
        ),
        position: type_token.position.clone(),
    })
}

pub fn parse_type_supers(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengPositionedExpression>, PengError> {
    let has_supers = match ptokens.peek() {
        Some(token) => {
            match &token.value {
                PengToken::Colon => true,
                _ => false,
            }
        }
        None => false,
    };

    if !has_supers {
        return Ok(Vec::new());
    }

    ptokens.next();
    let mut supers = Vec::new();

    loop {
        let super_expression = match parse_type_super_expression(ptokens) {
            Ok(expression) => expression,
            Err(e) => return Err(e),
        };

        supers.push(super_expression);

        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_message(
                    "expected type body".to_string(),
                ));
            }
        };

        match &token.value {
            PengToken::Comma => {
                ptokens.next();
            }
            PengToken::LeftCurlyBrace => break,
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or '{'".to_string(),
                    token.position.clone(),
                ));
            }
        }
    }

    Ok(supers)
}

fn parse_type_super_expression(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedExpression, PengError> {
    let mut lookahead = ptokens.clone();
    let mut tokens = Vec::new();
    let mut parenthesis_depth = 0usize;
    let mut bracket_depth = 0usize;

    loop {
        let token = match lookahead.next() {
            Some(token) => token,
            None => break,
        };

        let is_delimiter = if parenthesis_depth == 0 && bracket_depth == 0 {
            match &token.value {
                PengToken::Comma | PengToken::LeftCurlyBrace => true,
                _ => false,
            }
        } else {
            false
        };

        if is_delimiter {
            break;
        }

        match &token.value {
            PengToken::LeftParenthesis => {
                parenthesis_depth += 1;
            }
            PengToken::RightParenthesis => {
                if parenthesis_depth > 0 {
                    parenthesis_depth -= 1;
                }
            }
            PengToken::LeftBracket => {
                bracket_depth += 1;
            }
            PengToken::RightBracket => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            _ => {}
        }

        tokens.push(token.clone());
    }

    if tokens.is_empty() {
        let position = match ptokens.peek() {
            Some(token) => token.position.clone(),
            None => {
                return Err(PengError::new_message(
                    "expected super type".to_string(),
                ));
            }
        };

        return Err(PengError::new_positioned_message(
            "expected super type".to_string(),
            position,
        ));
    }

    let mut super_tokens = tokens.iter().peekable();
    let expression = match parse_expression(&mut super_tokens) {
        Ok(expression) => expression,
        Err(e) => return Err(e),
    };

    let remaining = match super_tokens.peek() {
        Some(token) => Some((*token).clone()),
        None => None,
    };

    match remaining {
        Some(token) => {
            return Err(PengError::new_positioned_message(
                "expected ',' or '{'".to_string(),
                token.position.clone(),
            ));
        }
        None => {}
    }

    let mut consumed = 0usize;

    while consumed < tokens.len() {
        match ptokens.next() {
            Some(_) => {
                consumed += 1;
            }
            None => {
                return Err(PengError::new_positioned_message(
                    "expected super type".to_string(),
                    expression.position.clone(),
                ));
            }
        }
    }

    Ok(expression)
}

pub fn parse_type_members(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<
    (
        Vec<PengPositionedVariableDeclaration>,
        Vec<PengPositionedFunctionDeclaration>,
    ),
    PengError,
> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected type body".to_string(),
            ));
        }
    };

    match &open_token.value {
        PengToken::LeftCurlyBrace => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '{'".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let mut fields = Vec::new();
    let mut functions = Vec::new();

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected '}'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &token.value {
            PengToken::RightCurlyBrace => {
                ptokens.next();
                break;
            }
            PengToken::Var => {
                let statement = match parse_variable_declaration_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => return Err(e),
                };

                match statement.value {
                    PengStatement::Declaration(PengDeclaration::Variable(field)) => {
                        fields.push(field);
                    }
                    _ => {
                        return Err(PengError::new_positioned_message(
                            "expected type field".to_string(),
                            statement.position,
                        ));
                    }
                }
            }
            PengToken::Func => {
                let statement = match parse_function_declaration_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => return Err(e),
                };

                match statement.value {
                    PengStatement::Declaration(PengDeclaration::Function(function)) => {
                        functions.push(function);
                    }
                    _ => {
                        return Err(PengError::new_positioned_message(
                            "expected type function".to_string(),
                            statement.position,
                        ));
                    }
                }
            }
            _ => {
                return Err(PengError::new_positioned_message(
                    "type body only accepts variable and function declarations".to_string(),
                    token.position.clone(),
                ));
            }
        }
    }

    Ok((fields, functions))
}
