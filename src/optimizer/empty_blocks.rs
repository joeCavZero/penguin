use crate::core::*;
use crate::parser::*;

pub fn remove_empty_block_statements(
    statements: Vec<PengPositionedStatement>,
) -> Result<Vec<PengPositionedStatement>, PengError> {
    let mut cleaned = Vec::new();

    for statement in statements {
        if statement_is_empty_block(&statement) {
            continue;
        }

        cleaned.push(statement);
    }

    Ok(cleaned)
}

pub fn empty_statements_to_none(
    statements: Option<Vec<PengPositionedStatement>>,
) -> Result<Option<Vec<PengPositionedStatement>>, PengError> {
    match statements {
        Some(statements) => {
            if statements.is_empty() {
                Ok(None)
            } else {
                Ok(Some(statements))
            }
        }

        None => Ok(None),
    }
}

pub fn statement_is_empty_block(statement: &PengPositionedStatement) -> bool {
    match &statement.value {
        PengStatement::Block(statements) => statements.is_empty(),
        _ => false,
    }
}

pub fn statements_are_empty(statements: &Vec<PengPositionedStatement>) -> bool {
    statements.is_empty()
}
