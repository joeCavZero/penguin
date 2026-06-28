use crate::core::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::parser::parse_expression::*;
use crate::parser::parse_type_expression::*;

pub fn parse_function_params(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengPositionedFunctionCallArg>, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected '('".to_string()));
        }
    };

    match &open_token.value {
        PengToken::LeftParenthesis => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '('".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let mut args = Vec::new();
    let mut has_variadic = false;

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ')'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &token.value {
            PengToken::RightParenthesis => {
                ptokens.next();
                break;
            }
            _ => {}
        }

        let expression = match parse_expression(ptokens) {
            Ok(expression) => expression,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while parsing parse_function_params".to_string(),
                )));
            }
        };

        let mut variadic = false;

        match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::TripleDot => {
                    let triple_dot_token = match ptokens.next() {
                        Some(token) => token,
                        None => {
                            return Err(PengError::new_positioned_message(
                                "expected '...'".to_string(),
                                expression.position.clone(),
                            ));
                        }
                    };

                    if has_variadic {
                        return Err(PengError::new_positioned_message(
                            "cannot use more than one variadic unpacking in the same function call"
                                .to_string(),
                            triple_dot_token.position.clone(),
                        ));
                    }

                    has_variadic = true;
                    variadic = true;
                }

                _ => {}
            },
            None => {}
        }

        let position = expression.position.clone();

        args.push(PengPositioned {
            value: PengFunctionCallArg {
                expression,
                variadic,
            },
            position,
        });

        let separator = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ')'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &separator.value {
            PengToken::Comma => {
                if variadic {
                    return Err(PengError::new_positioned_message(
                        "variadic unpacking must be the last argument".to_string(),
                        separator.position.clone(),
                    ));
                }
            }

            PengToken::RightParenthesis => break,

            _ => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ')'".to_string(),
                    separator.position.clone(),
                ));
            }
        }
    }

    Ok(args)
}

pub fn parse_function_params_declaration(
    ptokens: &mut PengPeekablePositionedToken,
) -> Result<Vec<PengPositionedFunctionParam>, PengError> {
    let open_token = match ptokens.next() {
        Some(token) => token,
        None => {
            return Err(PengError::SyntaxError("expected '('".to_string()));
        }
    };

    match &open_token.value {
        PengToken::LeftParenthesis => {}
        _ => {
            return Err(PengError::new_positioned_message(
                "expected '('".to_string(),
                open_token.position.clone(),
            ));
        }
    }

    let mut params = Vec::new();
    let mut has_variadic = false;

    loop {
        let token = match ptokens.peek() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ')'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &token.value {
            PengToken::RightParenthesis => {
                ptokens.next();
                break;
            }
            _ => {}
        }

        if has_variadic {
            return Err(PengError::new_positioned_message(
                "variadic parameter must be the last parameter".to_string(),
                token.position.clone(),
            ));
        }

        let name_token = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected parameter name".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        let name = match &name_token.value {
            PengToken::Identifier(name) => PengPositioned {
                value: name.clone(),
                position: name_token.position.clone(),
            },
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected parameter name".to_string(),
                    name_token.position.clone(),
                ));
            }
        };

        let variadic = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::TripleDot => true,
                _ => false,
            },
            None => false,
        };

        if variadic {
            ptokens.next();
            has_variadic = true;
        }

        let has_type_hint = match ptokens.peek() {
            Some(token) => match &token.value {
                PengToken::Colon => true,
                _ => false,
            },
            None => false,
        };

        let type_hint = if has_type_hint {
            ptokens.next();

            match parse_type_expression(ptokens) {
                Ok(type_expression) => Some(type_expression),
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while parsing parse_function_params".to_string(),
                    )));
                }
            }
        } else {
            None
        };

        params.push(PengPositioned {
            value: PengFunctionParam {
                name,
                type_hint,
                variadic,
            },
            position: name_token.position.clone(),
        });

        let separator = match ptokens.next() {
            Some(token) => token,
            None => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ')'".to_string(),
                    open_token.position.clone(),
                ));
            }
        };

        match &separator.value {
            PengToken::Comma => {}
            PengToken::RightParenthesis => break,
            _ => {
                return Err(PengError::new_positioned_message(
                    "expected ',' or ')'".to_string(),
                    separator.position.clone(),
                ));
            }
        }
    }

    Ok(params)
}
