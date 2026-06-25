use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    literal: &PengPositionedLiteral,
) -> Result<(), PengError> {
    match &literal.value {
        PengLiteral::Type(type_literal) => {
            return generate_type_literal(env, globals, context, type_literal);
        }
        PengLiteral::Module(module) => {
            return generate_module_literal(env, globals, context, module);
        }
        PengLiteral::Vector(values) => {
            return generate_vector_literal(env, globals, context, values);
        }
        PengLiteral::Object(fields) => {
            return generate_object_literal(env, globals, context, fields);
        }
        _ => {}
    }

    let value = match &literal.value {
        PengLiteral::Nil => PengHeapValue::Nil,
        PengLiteral::Int(value) => PengHeapValue::Int(*value),
        PengLiteral::Uint(value) => PengHeapValue::Uint(*value),
        PengLiteral::Byte(value) => PengHeapValue::Byte(*value),
        PengLiteral::Float32(value) => PengHeapValue::Float32(*value),
        PengLiteral::Float64(value) => PengHeapValue::Float64(*value),
        PengLiteral::Bool(value) => PengHeapValue::Bool(*value),
        PengLiteral::String(value) => PengHeapValue::String(value.clone()),
        PengLiteral::Type(_) => unreachable!(),
        PengLiteral::Function(function) => {
            match generate_function_value(env, globals, &function.params, &function.body) {
                Ok(value) => value,
                Err(e) => return Err(e),
            }
        }
        PengLiteral::Module(_) => unreachable!(),
        PengLiteral::Vector(_) => unreachable!(),
        PengLiteral::Operation(operation) => {
            match generate_operation_value(env, globals, &operation.params, &operation.body) {
                Ok(value) => value,
                Err(e) => return Err(e),
            }
        }
        PengLiteral::Object(_) => unreachable!(),
    };

    context.push_const_and_const_instruction(value);
    Ok(())
}

pub fn generate_type_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
) -> Result<(), PengError> {
    match generate_type_literal_after_base(env, globals, context, literal) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_object_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    fields: &[PengObjectFieldLiteral],
) -> Result<(), PengError> {
    context.bytecode.push(PengInstruction::CreateEmptyObject);

    for field in fields {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(field.name.value.clone());

        match generate_expression(env, globals, context, &field.value) {
            Ok(()) => {}
            Err(e) => return Err(e),
        };
        context
            .bytecode
            .push(PengInstruction::SetConstAttribute(name));
    }

    Ok(())
}

pub fn generate_type_literal_after_base(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
) -> Result<(), PengError> {
    for super_type in &literal.supers {
        match generate_expression(env, globals, context, super_type) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    context
        .bytecode
        .push(PengInstruction::CreateSuperType(literal.supers.len()));

    for field in &literal.fields {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(field.value.name.value.clone());

        match &field.value.value {
            Some(value) => match generate_expression(env, globals, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            },
            None => context.push_const_and_const_instruction(PengHeapValue::Nil),
        }

        context
            .bytecode
            .push(PengInstruction::SetConstAttribute(name));
    }

    for function in &literal.functions {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.ensure_pooled_name_ptr(function.value.name.value.clone());

        let value = match generate_function_declaration_value(env, globals, function) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        context.push_const_and_const_instruction(value);

        context
            .bytecode
            .push(PengInstruction::SetConstAttribute(name));
    }

    Ok(())
}

pub fn generate_vector_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    values: &[PengPositionedExpression],
) -> Result<(), PengError> {
    for value in values {
        match generate_expression(env, globals, context, value) {
            Ok(()) => {}
            Err(e) => return Err(e),
        };
    }

    context
        .bytecode
        .push(PengInstruction::CreateVector(values.len()));

    Ok(())
}

pub fn generate_module_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengHeapPtr>,
    context: &mut PengGeneratorContext,
    module: &PengModuleLiteral,
) -> Result<(), PengError> {
    context.bytecode.push(PengInstruction::CreateEmptyModule);

    for binded_declaration in &module.body {
        let declaration = match binded_declaration {
            PengBinded::Mutable(declaration) | PengBinded::Immutable(declaration) => declaration,
        };

        let name = match declaration {
            PengDeclaration::Var(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
        };

        let name_ptr = env.ensure_pooled_name_ptr(name);

        context.bytecode.push(PengInstruction::Duplicate);

        match declaration {
            PengDeclaration::Var(declaration) => match &declaration.value.value {
                Some(value) => match generate_expression(env, globals, context, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                None => context.push_const_and_const_instruction(PengHeapValue::Nil),
            },

            PengDeclaration::As(declaration) => {
                match generate_expression(env, globals, context, &declaration.value.value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                };
            }

            PengDeclaration::Function(declaration) => {
                let value = match generate_function_declaration_value(env, globals, declaration) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                context.push_const_and_const_instruction(value);
            }

            PengDeclaration::Type(declaration) => match &declaration.value.value {
                Some(value) => match generate_type_expression(env, globals, context, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                None => {
                    let literal = PengTypeLiteral {
                        supers: declaration.value.supers.clone(),
                        fields: declaration.value.fields.clone(),
                        functions: declaration.value.functions.clone(),
                    };

                    match generate_type_literal(env, globals, context, &literal) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    };
                }
            },

            PengDeclaration::Module(declaration) => {
                match generate_module_declaration_value(env, globals, context, declaration) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                };
            }

            PengDeclaration::Operation(declaration) => {
                let value = match generate_operation_declaration_value(env, globals, declaration) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                context.push_const_and_const_instruction(value);
            }
        }

        context
            .bytecode
            .push(PengInstruction::SetConstMember(name_ptr));
    }

    Ok(())
}
