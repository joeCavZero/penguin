use std::collections::HashMap;

use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_script(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengValuePtr, PengError> {
    let statements = match ast {
        PengAST::Script(statements) => statements,
        PengAST::Program(_) => {
            return Err(PengError::new_message(
                "expected script AST".to_string(),
            ));
        }
    };

    let mut globals = HashMap::new();
    let mut context = PengGeneratorContext::new();

    match generate_statements(
        env,
        &mut globals,
        &mut context,
        statements,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let script_function = create_anonymous_bytecode_function(
        env,
        context,
    );

    Ok(script_function)
}
