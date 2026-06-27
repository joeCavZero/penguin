use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_ast(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengHeapPtr, PengError> {
    match ast {
        PengAST::Script(statements) => {
            match generate_script(env, statements) {
                Ok(ptr) => Ok(
                    ptr
                ),
                Err(e) => Err(e),
            }
        }
        PengAST::Program(declarations) => {
            generate_program(env, declarations)
        }
    }
}