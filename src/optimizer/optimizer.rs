use crate::core::*;
use crate::parser::*;

use crate::optimizer::boolean_simplification;
use crate::optimizer::config::PengOptimizerConfig;
use crate::optimizer::constant_folding;
use crate::optimizer::constant_propagation;
use crate::optimizer::dead_code;
use crate::optimizer::empty_blocks;

pub fn optimize_ast(ast: PengAST, config: PengOptimizerConfig) -> Result<PengAST, PengError> {
    let mut constants = constant_propagation::PengConstantPropagationScope::new();

    match ast {
        PengAST::Program(declarations) => {
            let declarations = match optimize_declarations(declarations, &config, &mut constants) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while optimizing ast program".to_string(),
                    )));
                }
            };

            Ok(PengAST::Program(declarations))
        }

        PengAST::Script(statements) => {
            let statements = match optimize_statements(statements, &config, &mut constants) {
                Ok(v) => v,
                Err(e) => {
                    return Err(e.push(PengError::SyntaxError(
                        "failed while optimizing ast script".to_string(),
                    )));
                }
            };

            Ok(PengAST::Script(statements))
        }
    }
}

fn optimize_declarations(
    declarations: Vec<PengBindedDeclaration>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengBindedDeclaration>, PengError> {
    let mut optimized = Vec::new();

    for declaration in declarations {
        let declaration = match optimize_binded_declaration(declaration, config, constants) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.push(PengError::SyntaxError(
                    "failed while optimizing declarations".to_string(),
                )));
            }
        };

        match record_declaration_effects(&declaration, config, constants) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }

        optimized.push(declaration);
    }

    Ok(optimized)
}

fn optimize_binded_declaration(
    declaration: PengBindedDeclaration,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengBindedDeclaration, PengError> {
    match declaration {
        PengBinded::Mutable(declaration) => {
            let declaration = match optimize_declaration(declaration, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengBinded::Mutable(declaration))
        }

        PengBinded::Immutable(declaration) => {
            let declaration = match optimize_declaration(declaration, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengBinded::Immutable(declaration))
        }
    }
}

fn optimize_declaration(
    declaration: PengDeclaration,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengDeclaration, PengError> {
    match declaration {
        PengDeclaration::Var(declaration) => {
            let value = match declaration.value.value {
                Some(expression) => {
                    let expression = match optimize_expression(expression, config, constants) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Some(expression)
                }

                None => None,
            };

            Ok(PengDeclaration::Var(PengPositioned {
                position: declaration.position,
                value: PengVariableDeclaration {
                    name: declaration.value.name,
                    type_hint: declaration.value.type_hint,
                    value,
                },
            }))
        }

        PengDeclaration::As(declaration) => {
            let value = match optimize_expression(declaration.value.value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengDeclaration::As(PengPositioned {
                position: declaration.position,
                value: PengAsDeclaration {
                    name: declaration.value.name,
                    type_hint: declaration.value.type_hint,
                    value,
                },
            }))
        }

        PengDeclaration::Function(declaration) => {
            let body = match optimize_function_body(declaration.value.body, config) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengDeclaration::Function(PengPositioned {
                position: declaration.position,
                value: PengFunctionDeclaration {
                    name: declaration.value.name,
                    params: declaration.value.params,
                    return_type: declaration.value.return_type,
                    body,
                },
            }))
        }

        PengDeclaration::Type(declaration) => {
            let fields =
                match optimize_variable_declarations(declaration.value.fields, config, constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

            let functions =
                match optimize_function_declarations(declaration.value.functions, config) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

            Ok(PengDeclaration::Type(PengPositioned {
                position: declaration.position,
                value: PengTypeDeclaration {
                    name: declaration.value.name,
                    value: declaration.value.value,
                    supers: declaration.value.supers,
                    fields,
                    functions,
                },
            }))
        }

        PengDeclaration::Module(declaration) => {
            let mut module_constants = constant_propagation::PengConstantPropagationScope::new();

            let body = match optimize_declarations(
                declaration.value.body,
                config,
                &mut module_constants,
            ) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengDeclaration::Module(PengPositioned {
                position: declaration.position,
                value: PengModuleDeclaration {
                    name: declaration.value.name,
                    body,
                },
            }))
        }

        PengDeclaration::Operation(declaration) => {
            let body = match optimize_function_body(declaration.value.body, config) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengDeclaration::Operation(PengPositioned {
                position: declaration.position,
                value: PengOperationDeclaration {
                    name: declaration.value.name,
                    params: declaration.value.params,
                    return_type: declaration.value.return_type,
                    body,
                },
            }))
        }
    }
}

fn optimize_variable_declarations(
    declarations: Vec<PengBinded<PengPositionedVariableDeclaration>>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengBinded<PengPositionedVariableDeclaration>>, PengError> {
    let mut optimized = Vec::new();

    for binded_declaration in declarations {
        let (declaration, immutable) = match binded_declaration {
            PengBinded::Mutable(declaration) => (declaration, false),
            PengBinded::Immutable(declaration) => (declaration, true),
        };
        let value = match declaration.value.value {
            Some(expression) => {
                let expression = match optimize_expression(expression, config, constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                Some(expression)
            }

            None => None,
        };

        let declaration = PengPositioned {
            position: declaration.position,
            value: PengVariableDeclaration {
                name: declaration.value.name,
                type_hint: declaration.value.type_hint,
                value,
            },
        };
        optimized.push(if immutable {
            PengBinded::Immutable(declaration)
        } else {
            PengBinded::Mutable(declaration)
        });
    }

    Ok(optimized)
}

fn optimize_function_declarations(
    declarations: Vec<PengBinded<PengPositionedFunctionDeclaration>>,
    config: &PengOptimizerConfig,
) -> Result<Vec<PengBinded<PengPositionedFunctionDeclaration>>, PengError> {
    let mut optimized = Vec::new();

    for binded_declaration in declarations {
        let (declaration, immutable) = match binded_declaration {
            PengBinded::Mutable(declaration) => (declaration, false),
            PengBinded::Immutable(declaration) => (declaration, true),
        };
        let body = match optimize_function_body(declaration.value.body, config) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let declaration = PengPositioned {
            position: declaration.position,
            value: PengFunctionDeclaration {
                name: declaration.value.name,
                params: declaration.value.params,
                return_type: declaration.value.return_type,
                body,
            },
        };
        optimized.push(if immutable {
            PengBinded::Immutable(declaration)
        } else {
            PengBinded::Mutable(declaration)
        });
    }

    Ok(optimized)
}

fn optimize_function_body(
    body: Vec<PengPositionedStatement>,
    config: &PengOptimizerConfig,
) -> Result<Vec<PengPositionedStatement>, PengError> {
    /*
        Conservador:
        função começa com escopo novo de propagação.
        Assim constante global não substitui parâmetro com mesmo nome.
    */
    let mut constants = constant_propagation::PengConstantPropagationScope::new();

    optimize_statements(body, config, &mut constants)
}

fn optimize_statements(
    statements: Vec<PengPositionedStatement>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengPositionedStatement>, PengError> {
    let mut optimized = Vec::new();

    for statement in statements {
        let statement = match optimize_statement(statement, config, constants) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match record_statement_effects(&statement, config, constants) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }

        let stops_flow = if config.dead_code {
            dead_code::statement_stops_flow(&statement)
        } else {
            false
        };

        optimized.push(statement);

        if stops_flow {
            break;
        }
    }

    if config.dead_code {
        optimized = match dead_code::remove_unreachable_statements(optimized) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    if config.empty_blocks {
        optimized = match empty_blocks::remove_empty_block_statements(optimized) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    Ok(optimized)
}

fn optimize_statement(
    statement: PengPositionedStatement,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengPositionedStatement, PengError> {
    let position = statement.position.clone();

    let value = match statement.value {
        PengStatement::Block(statements) => {
            let mut block_constants = constants.clone();
            block_constants.push_frame();

            let statements = match optimize_statements(statements, config, &mut block_constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::Block(statements)
        }

        PengStatement::Declaration(declaration) => {
            let declaration = match optimize_binded_declaration(declaration, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::Declaration(declaration)
        }

        PengStatement::Return(value) => {
            let value = match value {
                Some(expression) => {
                    let expression = match optimize_expression(expression, config, constants) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Some(expression)
                }

                None => None,
            };

            PengStatement::Return(value)
        }

        PengStatement::If(statement) => {
            let condition = match optimize_expression(statement.condition, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let mut then_constants = constants.clone();
            then_constants.push_frame();

            let then_branch =
                match optimize_statements(statement.then_branch, config, &mut then_constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

            let else_branch = match statement.else_branch {
                Some(statements) => {
                    let mut else_constants = constants.clone();
                    else_constants.push_frame();

                    let statements =
                        match optimize_statements(statements, config, &mut else_constants) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                    Some(statements)
                }

                None => None,
            };

            let else_branch = if config.empty_blocks {
                match empty_blocks::empty_statements_to_none(else_branch) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                }
            } else {
                else_branch
            };

            PengStatement::If(PengIfStatement {
                condition,
                then_branch,
                else_branch,
            })
        }

        PengStatement::Match(statement) => {
            let value = match optimize_expression(statement.value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let mut arms = Vec::new();

            for arm in statement.arms {
                let pattern = match optimize_expression(arm.pattern, config, constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                let mut arm_constants = constants.clone();
                arm_constants.push_frame();

                let body = match optimize_statements(arm.body, config, &mut arm_constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                arms.push(PengMatchArm { pattern, body });
            }

            let elsing = match statement.elsing {
                Some(statements) => {
                    let mut else_constants = constants.clone();
                    else_constants.push_frame();

                    let statements =
                        match optimize_statements(statements, config, &mut else_constants) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                    Some(statements)
                }

                None => None,
            };

            let elsing = if config.empty_blocks {
                match empty_blocks::empty_statements_to_none(elsing) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                }
            } else {
                elsing
            };

            PengStatement::Match(PengMatchStatement {
                value,
                arms,
                elsing,
            })
        }

        PengStatement::While(statement) => {
            let condition = match optimize_expression(statement.condition, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let mut body_constants = constants.clone();
            body_constants.push_frame();

            let body = match optimize_statements(statement.body, config, &mut body_constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::While(PengWhileStatement { condition, body })
        }

        PengStatement::For(statement) => {
            let mut for_constants = constants.clone();
            for_constants.push_frame();

            let initializer = match statement.initializer {
                Some(statement) => {
                    let statement = match optimize_statement(*statement, config, &mut for_constants)
                    {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    match record_statement_effects(&statement, config, &mut for_constants) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }

                    Some(Box::new(statement))
                }

                None => None,
            };

            let condition = match statement.condition {
                Some(expression) => {
                    let expression =
                        match optimize_expression(expression, config, &mut for_constants) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                    Some(expression)
                }

                None => None,
            };

            let increment = match statement.increment {
                Some(statement) => {
                    let statement = match optimize_statement(*statement, config, &mut for_constants)
                    {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Some(Box::new(statement))
                }

                None => None,
            };

            let mut body_constants = for_constants.clone();
            body_constants.push_frame();

            let body = match optimize_statements(statement.body, config, &mut body_constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::For(PengForStatement {
                initializer,
                condition,
                increment,
                body,
            })
        }

        PengStatement::Loop(statements) => {
            let mut loop_constants = constants.clone();
            loop_constants.push_frame();

            let statements = match optimize_statements(statements, config, &mut loop_constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::Loop(statements)
        }

        PengStatement::Assign(assignment) => {
            let assignment = match optimize_assignment(assignment, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::Assign(assignment)
        }

        PengStatement::Expression(expression) => {
            let expression = match optimize_expression(expression, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengStatement::Expression(expression)
        }

        PengStatement::Break => PengStatement::Break,
        PengStatement::Continue => PengStatement::Continue,
    };

    Ok(PengPositioned { position, value })
}

fn optimize_assignment(
    assignment: PengAssignStatement,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengAssignStatement, PengError> {
    match assignment {
        PengAssignStatement::Assign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::Assign { target, value })
        }

        PengAssignStatement::AddAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::AddAssign { target, value })
        }

        PengAssignStatement::SubtractAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::SubtractAssign { target, value })
        }

        PengAssignStatement::MultiplyAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::MultiplyAssign { target, value })
        }

        PengAssignStatement::DivideAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::DivideAssign { target, value })
        }

        PengAssignStatement::PowerAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::PowerAssign { target, value })
        }

        PengAssignStatement::RemainderAssign { target, value } => {
            let target = match optimize_assignment_target(target, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let value = match optimize_expression(value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengAssignStatement::RemainderAssign { target, value })
        }
    }
}

fn optimize_assignment_target(
    target: PengPositionedExpression,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengPositionedExpression, PengError> {
    let position = target.position.clone();

    match target.value {
        PengExpression::Identifier(name) => {
            /*
                Importante:
                não roda constant propagation no identificador raiz do target.
                Senão `x = 2` poderia virar `10 = 2`.
            */
            Ok(PengPositioned {
                position,
                value: PengExpression::Identifier(name),
            })
        }

        PengExpression::AttributeAccess(access) => {
            let object = match optimize_expression(*access.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                position,
                value: PengExpression::AttributeAccess(PengAttributeAccessExpression {
                    object: Box::new(object),
                    name: access.name,
                }),
            })
        }

        PengExpression::MemberAccess(access) => {
            let object = match optimize_expression(*access.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                position,
                value: PengExpression::MemberAccess(PengMemberAccessExpression {
                    object: Box::new(object),
                    name: access.name,
                }),
            })
        }

        PengExpression::Index(index) => {
            let object = match optimize_expression(*index.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let index_value = match optimize_expression(*index.index, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            Ok(PengPositioned {
                position,
                value: PengExpression::Index(PengIndexExpression {
                    object: Box::new(object),
                    index: Box::new(index_value),
                }),
            })
        }

        value => Ok(PengPositioned { position, value }),
    }
}

fn optimize_expression(
    expression: PengPositionedExpression,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengPositionedExpression, PengError> {
    let position = expression.position.clone();

    let expression = match expression.value {
        PengExpression::Literal(literal) => {
            let literal = match optimize_literal(literal, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::Literal(literal),
            }
        }

        PengExpression::Identifier(name) => PengPositioned {
            position,
            value: PengExpression::Identifier(name),
        },

        PengExpression::Type(type_expression) => {
            /*
                Conservador:
                não otimiza dentro de type expression agora.
                Isso evita trocar nome de tipo por constante sem querer.
            */
            PengPositioned {
                position,
                value: PengExpression::Type(type_expression),
            }
        }

        PengExpression::Unary { operator, value } => {
            let value = match optimize_expression(*value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::Unary {
                    operator,
                    value: Box::new(value),
                },
            }
        }

        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            let left = match optimize_expression(*left, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let right = match optimize_expression(*right, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
            }
        }

        PengExpression::FuncCall(call) => {
            let function = match optimize_expression(*call.function, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let args = match optimize_call_args(call.args, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::FuncCall(PengFuncCallExpression {
                    function: Box::new(function),
                    args,
                }),
            }
        }

        PengExpression::MethodCall(call) => {
            let object = match optimize_expression(*call.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let args = match optimize_call_args(call.args, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::MethodCall(PengMethodCallExpression {
                    object: Box::new(object),
                    method: call.method,
                    args,
                }),
            }
        }

        PengExpression::AttributeAccess(access) => {
            let object = match optimize_expression(*access.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::AttributeAccess(PengAttributeAccessExpression {
                    object: Box::new(object),
                    name: access.name,
                }),
            }
        }

        PengExpression::MemberAccess(access) => {
            let object = match optimize_expression(*access.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::MemberAccess(PengMemberAccessExpression {
                    object: Box::new(object),
                    name: access.name,
                }),
            }
        }

        PengExpression::Index(index) => {
            let object = match optimize_expression(*index.object, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let index_value = match optimize_expression(*index.index, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::Index(PengIndexExpression {
                    object: Box::new(object),
                    index: Box::new(index_value),
                }),
            }
        }

        PengExpression::ObjectConstruction(construction) => {
            let object_type =
                match optimize_expression(*construction.object_type, config, constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

            let fields = match optimize_object_fields(construction.fields, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::ObjectConstruction(PengObjectConstructionExpression {
                    object_type: Box::new(object_type),
                    fields,
                }),
            }
        }

        PengExpression::OperationCall {
            left,
            operation,
            right,
        } => {
            let left = match optimize_expression(*left, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let operation = match optimize_expression(*operation, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let right = match optimize_expression(*right, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengPositioned {
                position,
                value: PengExpression::OperationCall {
                    left: Box::new(left),
                    operation: Box::new(operation),
                    right: Box::new(right),
                },
            }
        }

        PengExpression::Try { value, elsing } => {
            let value = match optimize_expression(*value, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let elsing = match elsing {
                Some(expression) => {
                    let expression = match optimize_expression(*expression, config, constants) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Some(Box::new(expression))
                }

                None => None,
            };

            PengPositioned {
                position,
                value: PengExpression::Try {
                    value: Box::new(value),
                    elsing,
                },
            }
        }
    };

    apply_expression_passes(expression, config, constants)
}

fn apply_expression_passes(
    expression: PengPositionedExpression,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengPositionedExpression, PengError> {
    let mut expression = expression;

    if config.constant_propagation {
        expression = match constants.replace_identifier(expression) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    if config.boolean_simplification {
        expression = match boolean_simplification::simplify_expression(expression) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    if config.constant_folding {
        expression = match constant_folding::fold_expression(expression) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    if config.boolean_simplification {
        expression = match boolean_simplification::simplify_expression(expression) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
    }

    Ok(expression)
}

fn optimize_call_args(
    args: Vec<PengPositionedFunctionCallArg>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengPositionedFunctionCallArg>, PengError> {
    let mut optimized = Vec::new();

    for arg in args {
        let expression = match optimize_expression(arg.value.expression, config, constants) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        optimized.push(PengPositioned {
            position: arg.position,
            value: PengFunctionCallArg {
                expression,
                variadic: arg.value.variadic,
            },
        });
    }

    Ok(optimized)
}

fn optimize_object_fields(
    fields: Vec<PengObjectFieldLiteral>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengObjectFieldLiteral>, PengError> {
    let mut optimized = Vec::new();

    for field in fields {
        let value = match optimize_expression(field.value, config, constants) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        optimized.push(PengObjectFieldLiteral {
            name: field.name,
            value,
        });
    }

    Ok(optimized)
}

fn optimize_literal(
    literal: PengPositionedLiteral,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<PengPositionedLiteral, PengError> {
    let position = literal.position.clone();

    let value = match literal.value {
        PengLiteral::Vector(values) => {
            let values = match optimize_expressions(values, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Vector(values)
        }

        PengLiteral::Object(fields) => {
            let fields = match optimize_object_fields(fields, config, constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Object(fields)
        }

        PengLiteral::Function(function) => {
            let body = match optimize_function_body(function.body, config) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Function(PengFunctionLiteral {
                params: function.params,
                return_type: function.return_type,
                body,
            })
        }

        PengLiteral::Module(module) => {
            let mut module_constants = constant_propagation::PengConstantPropagationScope::new();

            let body = match optimize_declarations(module.body, config, &mut module_constants) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Module(PengModuleLiteral { body })
        }

        PengLiteral::Operation(operation) => {
            let body = match optimize_function_body(operation.body, config) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Operation(PengOperationLiteral {
                params: operation.params,
                return_type: operation.return_type,
                body,
            })
        }

        PengLiteral::Type(type_literal) => {
            let fields =
                match optimize_variable_declarations(type_literal.fields, config, constants) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

            let functions = match optimize_function_declarations(type_literal.functions, config) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            PengLiteral::Type(PengTypeLiteral {
                supers: type_literal.supers,
                fields,
                functions,
            })
        }

        PengLiteral::Nil => PengLiteral::Nil,
        PengLiteral::Int(v) => PengLiteral::Int(v),
        PengLiteral::Uint(v) => PengLiteral::Uint(v),
        PengLiteral::Byte(v) => PengLiteral::Byte(v),
        PengLiteral::Float32(v) => PengLiteral::Float32(v),
        PengLiteral::Float64(v) => PengLiteral::Float64(v),
        PengLiteral::Bool(v) => PengLiteral::Bool(v),
        PengLiteral::String(v) => PengLiteral::String(v),
    };

    Ok(PengPositioned { position, value })
}

fn optimize_expressions(
    expressions: Vec<PengPositionedExpression>,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<Vec<PengPositionedExpression>, PengError> {
    let mut optimized = Vec::new();

    for expression in expressions {
        let expression = match optimize_expression(expression, config, constants) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        optimized.push(expression);
    }

    Ok(optimized)
}

fn record_statement_effects(
    statement: &PengPositionedStatement,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<(), PengError> {
    if !config.constant_propagation {
        return Ok(());
    }

    match &statement.value {
        PengStatement::Declaration(declaration) => {
            match constants.record_binded_declaration(declaration) {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        }

        PengStatement::Assign(assignment) => match constants.record_assignment(assignment) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        },

        _ => Ok(()),
    }
}

fn record_declaration_effects(
    declaration: &PengBindedDeclaration,
    config: &PengOptimizerConfig,
    constants: &mut constant_propagation::PengConstantPropagationScope,
) -> Result<(), PengError> {
    if !config.constant_propagation {
        return Ok(());
    }

    match constants.record_binded_declaration(declaration) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
