use std::collections::HashMap;

use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_script(
    env: &mut PengEnv,
    statements: &Vec<PengPositioned<PengStatement>>,
    global: &HashMap<usize, usize>,
) -> Result<PengHeapPtr, PengError> {
    let mut local_globals = global.clone();
    let mut context = PengGeneratorContext::new();

    match allocate_script_globals(env, statements, &mut local_globals) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match generate_statements(
        env,
        &mut local_globals,
        &mut context,
        statements,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let script_function = create_anonymous_bytecode_function(env, context);

    Ok(script_function)
}