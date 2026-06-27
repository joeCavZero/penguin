use crate::parser::*;
use crate::core::*;
use crate::generator::*;

pub fn generate_ast(
    env: &mut PengEnv,
    ast: &PengAST,
) -> Result<PengUnit, PengError> {
    let using_unit = PengUnit::library();

    generate_ast_using(env, ast, &using_unit)
}

pub fn generate_ast_using(
    env: &mut PengEnv,
    ast: &PengAST,
    using_unit: &PengUnit,
) -> Result<PengUnit, PengError> {
    match ast {
        PengAST::Script(statements) => {
            generate_script_using(env, statements, using_unit)
        }

        PengAST::Program(declarations) => {
            generate_program_using(env, declarations, using_unit)
        }
    }
}
