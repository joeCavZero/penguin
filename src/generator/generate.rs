use std::collections::HashMap;

use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_ast(
    env: &mut PengEnv,
    ast: &PengAST,
    globals: &HashMap<usize, usize>,
) -> Result<(HashMap<PengNamePoolPtr, PengHeapPtr>, PengHeapPtr), PengError> {
    match ast {
        PengAST::Script(statements) => {
            match generate_script(env, statements, globals) {
                Ok(ptr) => Ok(
                    (
                        HashMap::new(),
                        ptr,
                    )
                ),
                Err(e) => Err(e),
            }
        }
        PengAST::Program(declarations) => {
            generate_program(env, declarations, globals)
        }
    }
}