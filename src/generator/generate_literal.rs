use std::collections::HashMap;

use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    literal: &PengPositionedLiteral,
) -> Result<(), PengError> {
    match &literal.value {
        PengLiteral::Type(type_literal) => {
            return generate_type_literal(env, globals, context, type_literal);
        }
        PengLiteral::Module(_) => {
            return Err(PengError::new_positioned_message(
                "module literals require a module-construction bytecode instruction".to_string(),
                literal.position.clone(),
            ));
        }
        PengLiteral::Vector(_) => {
            return Err(PengError::new_positioned_message(
                "vector literals require a vector-construction bytecode instruction".to_string(),
                literal.position.clone(),
            ));
        }
        PengLiteral::Object(fields) => {
            return generate_object_literal(env, globals, context, fields);
        }
        _ => {}
    }

    let value = match &literal.value {
        PengLiteral::Nil => PengValue::Nil,
        PengLiteral::Int(value) => PengValue::Int(*value),
        PengLiteral::Uint(value) => PengValue::Uint(*value),
        PengLiteral::Byte(value) => PengValue::Byte(*value),
        PengLiteral::Float32(value) => PengValue::Float32(*value),
        PengLiteral::Float64(value) => PengValue::Float64(*value),
        PengLiteral::Bool(value) => PengValue::Bool(*value),
        PengLiteral::String(value) => PengValue::String(value.clone()),
        PengLiteral::Type(_) => unreachable!(),
        PengLiteral::Function(function) => {
            match generate_function_value(env, globals, &function.generics, &function.params, &function.body)
            {
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
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
) -> Result<(), PengError> {
    context.push_const_and_const_instruction(PengValue::Nil);

    match generate_type_literal_after_base(env, globals, context, literal) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_object_literal(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    fields: &[PengObjectFieldLiteral],
) -> Result<(), PengError> {
    context.push_const_and_const_instruction(PengValue::Type(PengType::Object));
    context.bytecode.push(PengInstruction::CreateObjectType);

    for field in fields {
        context.bytecode.push(PengInstruction::Duplicate);
        let name = env.get_pooled_name(field.name.value.clone());
        context
            .bytecode
            .push(PengInstruction::GetConstAttributeRef(name));
        match generate_expression(env, globals, context, &field.value) {
            Ok(()) => {},
            Err(e) => return Err(e),
        };
        context.bytecode.push(PengInstruction::StoreValue);
    }

    Ok(())
}

pub fn generate_type_literal_after_base(
    env: &mut PengEnv,
    globals: &mut HashMap<PengNamePoolPtr, PengValuePtr>,
    context: &mut PengGeneratorContext,
    literal: &PengTypeLiteral,
) -> Result<(), PengError> {
    if !literal.generics.is_empty() {
        return Err(PengError::new_positioned_message(
            "generic type literals are not supported by the current bytecode".to_string(),
            literal.generics[0].position.clone(),
        ));
    }

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

        let name = env.get_pooled_name(field.value.name.value.clone());

        context
            .bytecode
            .push(PengInstruction::GetConstAttributeRef(name));

        match &field.value.value {
            Some(value) => {
                match generate_expression(env, globals, context, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
            None => {
                context.push_const_and_const_instruction(PengValue::Nil);
            }
        }

        context.bytecode.push(PengInstruction::StoreValue);
    }

    for function in &literal.functions {
        context.bytecode.push(PengInstruction::Duplicate);

        let name = env.get_pooled_name(function.value.name.value.clone());

        context
            .bytecode
            .push(PengInstruction::GetConstAttributeRef(name));

        let value = match generate_function_declaration_value(env, globals, function) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        context.push_const_and_const_instruction(value);
        context.bytecode.push(PengInstruction::StoreValue);
    }

    Ok(())
}