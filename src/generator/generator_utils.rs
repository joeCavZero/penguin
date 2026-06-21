use std::collections::HashMap;

use crate::core::*;
use crate::parser::*;

pub struct PengGeneratorContext {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    locals: HashMap<String, usize>,
    next_local: usize,
}

impl PengGeneratorContext {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            consts: Vec::new(),
            locals: HashMap::new(),
            next_local: 0,
        }
    }

    pub fn push_const(
        &mut self,
        value: PengValue,
    ) {
        let const_index = self.consts.len();
        self.consts.push(value);
        self.bytecode.push(
            PengInstruction::PushConst(const_index),
        );
    }

    pub fn create_local(
        &mut self,
        name: String,
    ) -> usize {
        let local = self.next_local;
        self.next_local += 1;
        self.locals.insert(name, local);
        local
    }

    pub fn get_local(
        &self,
        name: &str,
    ) -> Option<usize> {
        match self.locals.get(name) {
            Some(local) => Some(*local),
            None => None,
        }
    }
}

pub fn create_anonymous_bytecode_function(
    env: &mut PengEnv,
    mut context: PengGeneratorContext,
) -> PengValuePtr {
    context.push_const(PengValue::Nil);
    context.bytecode.push(PengInstruction::Return);

    env.create_value(
        PengValue::Function(
            PengFunction::Bytecode(
                PengBytecodeFunction {
                    bytecode: context.bytecode,
                    consts: context.consts,
                    generics_count: 0,
                    using_values: Vec::new(),
                },
            ),
        ),
    )
}

pub fn generate_expression(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => {
            generate_literal(context, literal)
        }
        PengExpression::Identifier(identifier) => {
            generate_identifier(env, context, identifier)
        }
        PengExpression::Unary {
            operator,
            value,
        } => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match operator {
                PengUnaryOperator::Negate => {
                    context.bytecode.push(PengInstruction::Negate);
                }
                PengUnaryOperator::Not => {
                    context.bytecode.push(PengInstruction::Not);
                }
            }

            Ok(())
        }
        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            match generate_expression(env, context, left) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match generate_expression(env, context, right) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            generate_binary_operator(context, operator);
            Ok(())
        }
        PengExpression::Type(_) => {
            todo!("generate type value expression")
        }
        PengExpression::FuncCall(_) => {
            todo!("generate function call expression")
        }
        PengExpression::MethodCall(_) => {
            todo!("generate method call expression")
        }
        PengExpression::AttributeAccess(_) => {
            todo!("generate attribute access expression")
        }
        PengExpression::MemberAccess(_) => {
            todo!("generate member access expression")
        }
        PengExpression::Index(_) => {
            todo!("generate index expression")
        }
        PengExpression::ObjectConstruction(_) => {
            todo!("generate object construction expression")
        }
        PengExpression::OperationCall { .. } => {
            todo!("generate operation call expression")
        }
        PengExpression::Try { .. } => {
            todo!("generate try expression")
        }
    }
}

fn generate_literal(
    context: &mut PengGeneratorContext,
    literal: &PengPositionedLiteral,
) -> Result<(), PengError> {
    let value = match &literal.value {
        PengLiteral::Nil => PengValue::Nil,
        PengLiteral::Int(value) => PengValue::Int(*value),
        PengLiteral::Uint(value) => PengValue::Uint(*value),
        PengLiteral::Byte(value) => PengValue::Byte(*value),
        PengLiteral::Float32(value) => PengValue::Float32(*value),
        PengLiteral::Float64(value) => PengValue::Float64(*value),
        PengLiteral::Bool(value) => PengValue::Bool(*value),
        PengLiteral::String(value) => PengValue::String(value.clone()),
        PengLiteral::Type(_) => {
            todo!("generate type literal")
        }
        PengLiteral::Function(_) => {
            todo!("generate function literal")
        }
        PengLiteral::Module(_) => {
            todo!("generate module literal")
        }
        PengLiteral::Vector(_) => {
            todo!("generate vector literal")
        }
        PengLiteral::Operation(_) => {
            todo!("generate operation literal")
        }
        PengLiteral::Object(_) => {
            todo!("generate object literal")
        }
    };

    context.push_const(value);
    Ok(())
}

fn generate_identifier(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    identifier: &PengPositioned<String>,
) -> Result<(), PengError> {
    match context.get_local(&identifier.value) {
        Some(local) => {
            context.bytecode.push(
                PengInstruction::PushLocal(local),
            );
            return Ok(());
        }
        None => {}
    }

    match env.get_global(&identifier.value) {
        Some(value_ptr) => {
            context.bytecode.push(
                PengInstruction::PushValue(value_ptr),
            );
            Ok(())
        }
        None => Err(PengError::new_positioned_message(
            format!("unknown value '{}'", identifier.value),
            identifier.position.clone(),
        )),
    }
}

fn generate_binary_operator(
    context: &mut PengGeneratorContext,
    operator: &PengBinaryOperator,
) {
    let instruction = match operator {
        PengBinaryOperator::Add => PengInstruction::Add,
        PengBinaryOperator::Subtract => PengInstruction::Subtract,
        PengBinaryOperator::Multiply => PengInstruction::Multiply,
        PengBinaryOperator::Divide => PengInstruction::Divide,
        PengBinaryOperator::Power => PengInstruction::Power,
        PengBinaryOperator::Remainder => PengInstruction::Remainder,
        PengBinaryOperator::Concat => PengInstruction::Concat,
        PengBinaryOperator::And => PengInstruction::And,
        PengBinaryOperator::Or => PengInstruction::Or,
        PengBinaryOperator::Equals => PengInstruction::Equals,
        PengBinaryOperator::NotEquals => PengInstruction::NotEquals,
        PengBinaryOperator::GreaterThan => PengInstruction::GreaterThan,
        PengBinaryOperator::GreaterEqualsThan => {
            PengInstruction::GreaterEqualsThan
        }
        PengBinaryOperator::LessThan => PengInstruction::LessThan,
        PengBinaryOperator::LessEqualsThan => {
            PengInstruction::LessEqualsThan
        }
        PengBinaryOperator::As => {
            todo!("generate type conversion")
        }
    };

    context.bytecode.push(instruction);
}
