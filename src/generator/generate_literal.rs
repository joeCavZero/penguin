use crate::core::*;
use crate::parser::*;
use crate::generator::*;


pub fn generate_literal(
    env: &PengEnv,
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
        PengLiteral::Function(function) => {
            match generate_function_value(
                env,
                &function.generics,
                &function.params,
                &function.body,
            ) {
                Ok(value) => value,
                Err(e) => return Err(e),
            }
        }
        PengLiteral::Module(_) => {
            todo!("generate module literal")
        }
        PengLiteral::Vector(_) => {
            todo!("generate vector literal")
        }
        PengLiteral::Operation(operation) => {
            match generate_operation_value(
                env,
                &operation.params,
                &operation.body,
            ) {
                Ok(value) => value,
                Err(e) => return Err(e),
            }
        }
        PengLiteral::Object(_) => {
            todo!("generate object literal")
        }
    };

    context.push_const_and_const_instruction(value);
    Ok(())
}
