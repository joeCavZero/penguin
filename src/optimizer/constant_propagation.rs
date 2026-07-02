use std::collections::HashMap;

use crate::core::*;
use crate::parser::*;

#[derive(Debug, Clone)]
enum PengConstantPropagationEntry {
    Value(PengPositionedExpression),
    Blocked,
}

#[derive(Debug, Clone)]
pub struct PengConstantPropagationScope {
    frames: Vec<HashMap<String, PengConstantPropagationEntry>>,
}

impl PengConstantPropagationScope {
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
        }
    }

    pub fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    pub fn pop_frame(&mut self) -> Result<(), PengError> {
        if self.frames.len() <= 1 {
            return Err(PengError::SyntaxError(
                "cannot pop root constant propagation scope".to_string(),
            ));
        }

        match self.frames.pop() {
            Some(_) => Ok(()),
            None => Err(PengError::SyntaxError(
                "cannot pop empty constant propagation scope".to_string(),
            )),
        }
    }

    pub fn record_binded_declaration(
        &mut self,
        declaration: &PengBindedDeclaration,
    ) -> Result<(), PengError> {
        match declaration {
            PengBinded::Immutable(declaration) => self.record_immutable_declaration(declaration),

            PengBinded::Mutable(declaration) => self.record_mutable_declaration(declaration),
        }
    }

    pub fn record_assignment(&mut self, assignment: &PengAssignStatement) -> Result<(), PengError> {
        match assignment {
            PengAssignStatement::Assign { target, .. }
            | PengAssignStatement::AddAssign { target, .. }
            | PengAssignStatement::SubtractAssign { target, .. }
            | PengAssignStatement::MultiplyAssign { target, .. }
            | PengAssignStatement::DivideAssign { target, .. }
            | PengAssignStatement::PowerAssign { target, .. }
            | PengAssignStatement::RemainderAssign { target, .. } => {
                self.record_assignment_target(target)
            }
        }
    }

    pub fn replace_identifier(
        &self,
        expression: PengPositionedExpression,
    ) -> Result<PengPositionedExpression, PengError> {
        let position = expression.position.clone();

        let name = match expression.value {
            PengExpression::Identifier(name) => name,
            value => {
                return Ok(PengPositioned { position, value });
            }
        };

        let value = match self.lookup(&name.value) {
            Some(value) => value,
            None => {
                return Ok(PengPositioned {
                    position,
                    value: PengExpression::Identifier(name),
                });
            }
        };

        match clone_literal_expression_at_position(value, position) {
            Some(expression) => Ok(expression),
            None => Ok(PengPositioned {
                position: name.position.clone(),
                value: PengExpression::Identifier(name),
            }),
        }
    }

    fn record_immutable_declaration(
        &mut self,
        declaration: &PengDeclaration,
    ) -> Result<(), PengError> {
        match declaration {
            PengDeclaration::Var(declaration) => {
                let name = declaration.value.name.value.clone();

                match &declaration.value.value {
                    Some(expression) => {
                        if can_propagate_expression(expression) {
                            return self.set_current_value(name, expression.clone());
                        }

                        self.block_current_name(name)
                    }

                    None => self.block_current_name(name),
                }
            }

            PengDeclaration::As(declaration) => {
                let name = declaration.value.name.value.clone();

                if can_propagate_expression(&declaration.value.value) {
                    return self.set_current_value(name, declaration.value.value.clone());
                }

                self.block_current_name(name)
            }

            PengDeclaration::Function(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Type(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Module(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Operation(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }
        }
    }

    fn record_mutable_declaration(
        &mut self,
        declaration: &PengDeclaration,
    ) -> Result<(), PengError> {
        match declaration {
            PengDeclaration::Var(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::As(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Function(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Type(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Module(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }

            PengDeclaration::Operation(declaration) => {
                self.block_current_name(declaration.value.name.value.clone())
            }
        }
    }

    fn record_assignment_target(
        &mut self,
        target: &PengPositionedExpression,
    ) -> Result<(), PengError> {
        match &target.value {
            PengExpression::Identifier(name) => self.block_name_in_all_frames(name.value.clone()),

            /*
                Conservador:
                a[0] = ...
                obj.name = ...
                mod:name = ...

                Não sabemos qual símbolo isso pode afetar por alias/referência.
                Por enquanto não tentamos invalidar nada aqui.
            */
            PengExpression::AttributeAccess(_)
            | PengExpression::MemberAccess(_)
            | PengExpression::Index(_) => Ok(()),

            _ => Ok(()),
        }
    }

    fn set_current_value(
        &mut self,
        name: String,
        value: PengPositionedExpression,
    ) -> Result<(), PengError> {
        let frame = match self.frames.last_mut() {
            Some(frame) => frame,
            None => {
                return Err(PengError::SyntaxError(
                    "missing constant propagation scope".to_string(),
                ));
            }
        };

        frame.insert(name, PengConstantPropagationEntry::Value(value));
        Ok(())
    }

    fn block_current_name(&mut self, name: String) -> Result<(), PengError> {
        let frame = match self.frames.last_mut() {
            Some(frame) => frame,
            None => {
                return Err(PengError::SyntaxError(
                    "missing constant propagation scope".to_string(),
                ));
            }
        };

        frame.insert(name, PengConstantPropagationEntry::Blocked);
        Ok(())
    }

    fn block_name_in_all_frames(&mut self, name: String) -> Result<(), PengError> {
        if self.frames.is_empty() {
            return Err(PengError::SyntaxError(
                "missing constant propagation scope".to_string(),
            ));
        }

        for frame in &mut self.frames {
            frame.insert(name.clone(), PengConstantPropagationEntry::Blocked);
        }

        Ok(())
    }

    fn lookup(&self, name: &String) -> Option<PengPositionedExpression> {
        for frame in self.frames.iter().rev() {
            match frame.get(name) {
                Some(PengConstantPropagationEntry::Value(value)) => {
                    return Some(value.clone());
                }

                Some(PengConstantPropagationEntry::Blocked) => {
                    return None;
                }

                None => {}
            }
        }

        None
    }
}

pub fn can_propagate_expression(expression: &PengPositionedExpression) -> bool {
    match &expression.value {
        PengExpression::Literal(literal) => can_propagate_literal(&literal.value),

        _ => false,
    }
}

pub fn can_propagate_literal(literal: &PengLiteral) -> bool {
    match literal {
        PengLiteral::Nil => true,

        PengLiteral::Int(_) => true,
        PengLiteral::Uint(_) => true,
        PengLiteral::Byte(_) => true,

        PengLiteral::Float32(_) => true,
        PengLiteral::Float64(_) => true,

        PengLiteral::Bool(_) => true,
        PengLiteral::String(_) => true,

        PengLiteral::Type(_) => false,
        PengLiteral::Function(_) => false,
        PengLiteral::Module(_) => false,
        PengLiteral::Vector(_) => false,
        PengLiteral::Operation(_) => false,
        PengLiteral::Object(_) => false,
    }
}

fn clone_literal_expression_at_position(
    expression: PengPositionedExpression,
    position: PengPosition,
) -> Option<PengPositionedExpression> {
    let literal = match expression.value {
        PengExpression::Literal(literal) => literal.value,
        _ => return None,
    };

    if !can_propagate_literal(&literal) {
        return None;
    }

    Some(PengPositioned {
        position: position.clone(),
        value: PengExpression::Literal(PengPositioned {
            position,
            value: literal,
        }),
    })
}
