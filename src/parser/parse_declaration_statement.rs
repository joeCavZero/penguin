use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub fn parse_binded_declaration(
    tokens: &mut PengPeekablePositionedToken,
) -> Result<PengBindedDeclaration, PengError> {
    let is_const = matches!(tokens.peek().map(|t| &t.value), Some(PengToken::Const));

    if is_const {
        tokens.next();

        if matches!(tokens.peek().map(|t| &t.value), Some(PengToken::Var)) {
            return Err(PengError::new_positioned_message(
                "use `const name = value`, not `const var name = value`".to_string(),
                tokens.peek().unwrap().position.clone(),
            ));
        }
    }

    let declaration = if is_const {
        match parse_const_declaration(tokens) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_declaration_statement".to_string(),
                )));
            }
        }
    } else {
        match parse_mutable_declaration(tokens) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_declaration_statement".to_string(),
                )));
            }
        }
    };

    if is_const {
        Ok(PengBinded::Immutable(declaration))
    } else {
        Ok(PengBinded::Mutable(declaration))
    }
}

pub fn parse_variable_declaration_statement(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let var_token = match ptokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::new_message(
                "expected variable declaration".to_string(),
            ));
        }
    };

    match &var_token.value {
        PengToken::Var => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected 'var'".to_string(),
                var_token.position.clone(),
            ));
        }
    }

    let name = match ptokens.next() {
        Some(t) => match &t.value {
            PengToken::Identifier(name) => PengPositioned {
                value: name.clone(),
                position: t.position.clone(),
            },
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected variable name".to_string(),
                    t.position.clone(),
                ));
            }
        },
        None => {
            return Err(PengError::new_positioned_message(
                "expected variable name".to_string(),
                var_token.position.clone(),
            ));
        }
    };

    let mut type_hint = None;

    let has_colon = match ptokens.peek() {
        Some(t) => match &t.value {
            PengToken::Colon => true,
            _ => false,
        },
        None => false,
    };

    if has_colon {
        ptokens.next();

        match parse_type_expression(ptokens) {
            Ok(t) => {
                type_hint = Some(t);
            }
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_declaration_statement".to_string(),
                )));
            }
        }
    }

    let mut value = None;

    let has_equals = match ptokens.peek() {
        Some(t) => match &t.value {
            PengToken::Equals => true,
            _ => false,
        },
        None => false,
    };

    if has_equals {
        ptokens.next();

        match parse_expression(ptokens) {
            Ok(expr) => {
                value = Some(expr);
            }
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_declaration_statement".to_string(),
                )));
            }
        }
    }

    match consume_optional_semicolon(ptokens) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_declaration_statement".to_string(),
            )));
        }
    }

    let declaration = PengVariableDeclaration {
        name,
        type_hint,
        value,
    };

    let positioned_declaration = PengPositioned {
        value: declaration,
        position: var_token.position.clone(),
    };

    let stmt = PengStatement::Declaration(PengBinded::Mutable(PengDeclaration::Var(
        positioned_declaration,
    )));

    Ok(PengPositioned {
        value: stmt,
        position: var_token.position.clone(),
    })
}

pub fn parse_const_declaration(
    tokens: &mut PengPeekablePositionedToken,
) -> Result<PengDeclaration, PengError> {
    match tokens.peek().map(|t| &t.value) {
        Some(PengToken::Func) => {
            let statement = match parse_function_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Type) => {
            let statement = match parse_type_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Mod) => {
            let statement = match parse_module_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Oper) => {
            let statement = match parse_operation_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Union) => {
            let statement = match parse_union_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Identifier(_)) => {
            parse_const_variable_declaration(tokens).map(PengDeclaration::Var)
        }

        _ => Err(PengError::new_message(
            "expected const declaration".to_string(),
        )),
    }
}

fn parse_const_variable_declaration(
    tokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedVariableDeclaration, PengError> {
    let name_token = match tokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::new_message("expected const name".to_string()));
        }
    };

    let name = match &name_token.value {
        PengToken::Identifier(name) => PengPositioned {
            value: name.clone(),
            position: name_token.position.clone(),
        },
        _ => {
            return Err(PengError::new_positioned_message(
                "expected const name".to_string(),
                name_token.position.clone(),
            ));
        }
    };

    let mut type_hint = None;

    if matches!(tokens.peek().map(|t| &t.value), Some(PengToken::Colon)) {
        tokens.next();
        match parse_type_expression(tokens) {
            Ok(v) => {
                type_hint = Some(v);
            }
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_declaration_statement".to_string(),
                )));
            }
        }
    }

    let equals_token = match tokens.next() {
        Some(t) => t,
        None => {
            return Err(PengError::new_positioned_message(
                "expected '='".to_string(),
                name.position.clone(),
            ));
        }
    };

    match &equals_token.value {
        PengToken::Equals => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '='".to_string(),
                equals_token.position.clone(),
            ));
        }
    }

    let value = match parse_expression(tokens) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_declaration_statement".to_string(),
            )));
        }
    };

    Ok(PengPositioned {
        position: name.position.clone(),
        value: PengVariableDeclaration {
            name,
            type_hint,
            value: Some(value),
        },
    })
}

pub fn parse_declaration_statement(
    tokens: &mut PengPeekablePositionedToken,
) -> Result<PengPositionedStatement, PengError> {
    let position = match tokens.peek().map(|t| t.position.clone()) {
        Some(p) => p,
        None => return Err(PengError::new_message("expected declaration".to_string())),
    };

    let declaration = match parse_binded_declaration(tokens) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_declaration_statement".to_string(),
            )));
        }
    };

    match consume_optional_semicolon(tokens) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::SyntaxError(
                "failed while parsing parse_declaration_statement".to_string(),
            )));
        }
    }

    Ok(PengPositioned {
        value: PengStatement::Declaration(declaration),
        position,
    })
}

pub fn parse_mutable_declaration(
    tokens: &mut PengPeekablePositionedToken,
) -> Result<PengDeclaration, PengError> {
    match tokens.peek().map(|t| &t.value) {
        Some(PengToken::Var) => {
            let statement = match parse_variable_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Func) => {
            let statement = match parse_function_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Type) => {
            let statement = match parse_type_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Union) => {
            let statement = match parse_union_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Mod) => {
            let statement = match parse_module_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        Some(PengToken::Oper) => {
            let statement = match parse_operation_declaration_statement(tokens) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_declaration_statement".to_string(),
                    )));
                }
            };
            extract_declaration(statement)
        }

        _ => Err(PengError::new_message(
            "expected mutable declaration".to_string(),
        )),
    }
}

fn extract_declaration(statement: PengPositionedStatement) -> Result<PengDeclaration, PengError> {
    match statement.value {
        PengStatement::Declaration(PengBinded::Mutable(declaration))
        | PengStatement::Declaration(PengBinded::Immutable(declaration)) => Ok(declaration),

        _ => Err(PengError::new_positioned_message(
            "expected declaration".to_string(),
            statement.position,
        )),
    }
}

pub fn parse_declaration_body(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengBindedDeclaration>, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::new_message(
                "expected declaration body".to_string(),
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

    let mut declarations = Vec::new();

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
            PengToken::Var
            | PengToken::Const
            | PengToken::Func
            | PengToken::Type
            | PengToken::Union
            | PengToken::Mod
            | PengToken::Oper => {
                let statement = match parse_statement(ptokens) {
                    Ok(statement) => statement,
                    Err(e) => {
                        return Err(e.push(PengError::SyntaxError(
                            "failed while parsing parse_declaration_statement".to_string(),
                        )));
                    }
                };

                match statement.value {
                    PengStatement::Declaration(declaration) => {
                        declarations.push(declaration);
                    }
                    _ => {
                        return Err(PengError::new_positioned_message(
                            "module body only accepts declarations".to_string(),
                            statement.position,
                        ));
                    }
                }
            }
            _ => {
                return Err(PengError::new_positioned_message(
                    "module body only accepts declarations".to_string(),
                    token.position.clone(),
                ));
            }
        }
    }

    Ok(declarations)
}
