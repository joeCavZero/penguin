use crate::parser::*;
use crate::core::*;
pub use crate::generator::generate_program::*;
pub use crate::generator::generate_script::*;
pub use crate::generator::generator_utils::*;
pub use crate::generator::generate_expression::*;
pub use crate::generator::generate_literal::*;
pub use crate::generator::generate_function::*;
pub use crate::generator::generate_operation::*;
pub use crate::generator::generate_statement::*;
pub use crate::generator::generate_loops::*;


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
