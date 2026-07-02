use crate::core::*;
use crate::parser::*;

pub fn remove_unreachable_statements(
    statements: Vec<PengPositionedStatement>,
) -> Result<Vec<PengPositionedStatement>, PengError> {
    let mut reachable = Vec::new();
    let mut stopped = false;

    for statement in statements {
        if stopped {
            continue;
        }

        stopped = statement_stops_flow(&statement);
        reachable.push(statement);
    }

    Ok(reachable)
}

pub fn statement_stops_flow(statement: &PengPositionedStatement) -> bool {
    match &statement.value {
        PengStatement::Return(_) => true,

        PengStatement::Break => true,

        PengStatement::Continue => true,

        PengStatement::Block(statements) => block_stops_flow(statements),

        PengStatement::If(statement) => {
            let then_stops = block_stops_flow(&statement.then_branch);

            let else_stops = match &statement.else_branch {
                Some(statements) => block_stops_flow(statements),
                None => false,
            };

            then_stops && else_stops
        }

        PengStatement::Match(statement) => {
            if statement.arms.is_empty() {
                return false;
            }

            let elsing = match &statement.elsing {
                Some(statements) => statements,
                None => return false,
            };

            if !block_stops_flow(elsing) {
                return false;
            }

            for arm in &statement.arms {
                if !block_stops_flow(&arm.body) {
                    return false;
                }
            }

            true
        }

        PengStatement::While(_) => false,
        PengStatement::For(_) => false,
        PengStatement::Loop(_) => false,

        PengStatement::Declaration(_) => false,
        PengStatement::Assign(_) => false,
        PengStatement::Expression(_) => false,
    }
}

pub fn block_stops_flow(statements: &Vec<PengPositionedStatement>) -> bool {
    let last = match statements.last() {
        Some(statement) => statement,
        None => return false,
    };

    statement_stops_flow(last)
}
