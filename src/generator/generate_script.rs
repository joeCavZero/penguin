use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_script(
    env: &mut PengEnv,
    statements: &Vec<PengPositioned<PengStatement>>,
) -> Result<PengUnit, PengError> {
    let mut context = PengGeneratorContext::new();

    match generate_statements(env, &mut context, statements) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_script".to_string(),
            )));
        }
    }

    let script_function = create_anonymous_bytecode_function(
        env,
        context,
        PengBytecodeFunctionParams::Fixed(0),
    );

    Ok(PengUnit::empty(script_function))
}