use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_ast(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengUnit, PengError> {
    match ast {
        PengAST::Script(statements) => {
            generate_script(env, statements)
        }

        PengAST::Program(declarations) => {
            generate_program(env, declarations)
        }
    }
}