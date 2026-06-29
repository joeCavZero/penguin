use crate::core::*;
use crate::lexer::*;
use crate::parser::parser_utils::expect_identifier;
use crate::parser::parser::*;
use crate::parser::parse_declaration_statement::*;
use crate::parser::parse_expression::*;

pub fn parse_type_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let type_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError(
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
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_declaration_statement".to_string(),
            )));
        }
    };

    let supers = match parse_type_supers(ptokens) {
        Ok(supers) => supers,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_declaration_statement".to_string(),
            )));
        }
    };

    let (fields, functions) = match parse_type_members(ptokens) {
        Ok(members) => members,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_declaration_statement".to_string(),
            )));
        }
    };

    let declaration = PengPositioned {
        value: PengTypeDeclaration {
            name,
            value: None,
            supers,
            fields,
            functions,
        },
        position: type_token.position.clone(),
    };

    Ok(PengPositioned {
        value: PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Type(declaration))),
        position: type_token.position.clone(),
    })
}

pub fn parse_type_supers(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengPositionedExpression>, PengError> {
    let has_supers = match ptokens.peek() {
        Some(token) => match &token.value {
            PengToken::Colon => true,
            _ => false,
        },
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
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_type_declaration_statement".to_string(),
                )));
            }
        };

        supers.push(super_expression);

        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::SyntaxError("expected type body".to_string()));
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
                return Err(PengError::SyntaxError("expected super type".to_string()));
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
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_type_declaration_statement".to_string(),
            )));
        }
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
        Vec<PengBinded<PengPositionedVariableDeclaration>>,
        Vec<PengBinded<PengPositionedFunctionDeclaration>>,
    ),
    PengError,
> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected type body".to_string()));
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
            Some(token) => (*token).clone(),
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

            PengToken::Var | PengToken::Func | PengToken::Const => {
                let declaration = match parse_binded_declaration(ptokens) {
                    Ok(declaration) => declaration,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_type_declaration_statement".to_string(),
                        )));
                    }
                };

                match declaration {
                    PengBinded::Mutable(PengDeclaration::Var(field)) => {
                        fields.push(PengBinded::Mutable(field));
                    }
                    PengBinded::Immutable(PengDeclaration::Var(field)) => {
                        fields.push(PengBinded::Immutable(field));
                    }

                    PengBinded::Mutable(PengDeclaration::Function(function)) => {
                        functions.push(PengBinded::Mutable(function));
                    }
                    PengBinded::Immutable(PengDeclaration::Function(function)) => {
                        functions.push(PengBinded::Immutable(function));
                    }

                    _ => {
                        return Err(PengError::new_positioned_message(
                            "type body only accepts variable and function declarations".to_string(),
                            token.position.clone(),
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
