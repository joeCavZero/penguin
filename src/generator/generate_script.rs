use crate::core::*;
use crate::generator::*;
use crate::parser::*;

pub fn generate_script_using(
    env: &mut PengEnv,
    statements: &Vec<PengPositioned<PengStatement>>,
    using_unit: &PengUnit,
) -> Result<PengUnit, PengError> {
    let mut context = PengGeneratorContext::new();
    context.use_globals(env, using_unit.globals().clone());

    match generate_statements(env, &mut context, statements) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generate_script".to_string(),
            )));
        }
    }

    let terminal_pos = match statements.last() {
        Some(statement) => statement.position.clone(),
        // An empty script has no source node to associate with its synthetic return.
        None => PengPosition::new(0, 0, None),
    };
    let script_function = create_anonymous_bytecode_function(
        env,
        context,
        PengBytecodeFunctionParams::Fixed(0),
        terminal_pos,
    );

    Ok(PengUnit::empty(script_function))
}
