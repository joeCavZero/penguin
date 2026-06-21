use std::collections::HashMap;

use crate::core::*;
use crate::parser::*;

pub struct PengGeneratorContext {
    pub bytecode: Vec<PengInstruction>,
    pub consts: Vec<PengValue>,
    scopes: Vec<HashMap<String, usize>>,
    next_local: usize,
    loops: Vec<PengLoopContext>,
}

struct PengLoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
    continue_target: Option<usize>,
}

impl PengGeneratorContext {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            consts: Vec::new(),
            scopes: vec![HashMap::new()],
            next_local: 0,
            loops: Vec::new(),
        }
    }

    pub fn push_const_and_const_instruction(
        &mut self,
        value: PengValue,
    ) {
        let mut const_index = None;
        let mut index = 0usize;

        while index < self.consts.len() {
            let existing = match self.consts.get(index) {
                Some(existing) => existing,
                None => break,
            };

            if existing == &value {
                const_index = Some(index);
                break;
            }

            index += 1;
        }

        let const_index = match const_index {
            Some(const_index) => const_index,
            None => {
                let const_index = self.consts.len();
                self.consts.push(value);
                const_index
            }
        };

        self.bytecode.push(
            PengInstruction::PushConst(const_index),
        );
    }

    pub fn create_local(
        &mut self,
        name: String,
    ) -> usize {
        let local = self.next_local;
        self.next_local += 1;

        match self.scopes.last_mut() {
            Some(scope) => {
                scope.insert(name, local);
            }
            None => {
                let mut scope = HashMap::new();
                scope.insert(name, local);
                self.scopes.push(scope);
            }
        }

        local
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn get_local(
        &self,
        name: &str,
    ) -> Option<usize> {
        let mut index = self.scopes.len();

        while index > 0 {
            index -= 1;

            let scope = match self.scopes.get(index) {
                Some(scope) => scope,
                None => continue,
            };

            match scope.get(name) {
                Some(local) => return Some(*local),
                None => {}
            }
        }

        None
    }

    fn emit_jump(&mut self) -> usize {
        let instruction_index = self.bytecode.len();
        self.bytecode.push(PengInstruction::Jump(0));
        instruction_index
    }

    fn emit_jump_if_false(&mut self) -> usize {
        let instruction_index = self.bytecode.len();
        self.bytecode.push(PengInstruction::JumpIfFalse(0));
        instruction_index
    }

    fn patch_jump(
        &mut self,
        instruction_index: usize,
        target: usize,
    ) {
        let instruction = match self.bytecode.get_mut(instruction_index) {
            Some(instruction) => instruction,
            None => return,
        };

        match instruction {
            PengInstruction::Jump(destination)
            | PengInstruction::JumpIfTrue(destination)
            | PengInstruction::JumpIfFalse(destination) => {
                *destination = target;
            }
            _ => {}
        }
    }

    fn push_loop(
        &mut self,
        continue_target: Option<usize>,
    ) {
        self.loops.push(PengLoopContext {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            continue_target,
        });
    }

    fn pop_loop(&mut self) -> Option<PengLoopContext> {
        self.loops.pop()
    }

    fn emit_break(&mut self) -> bool {
        if self.loops.is_empty() {
            return false;
        }

        let jump = self.emit_jump();

        match self.loops.last_mut() {
            Some(loop_context) => {
                loop_context.break_jumps.push(jump);
                true
            }
            None => false,
        }
    }

    fn emit_continue(&mut self) -> bool {
        if self.loops.is_empty() {
            return false;
        }

        let jump = self.emit_jump();

        match self.loops.last_mut() {
            Some(loop_context) => {
                loop_context.continue_jumps.push(jump);
                true
            }
            None => false,
        }
    }

    fn patch_loop(
        &mut self,
        loop_context: PengLoopContext,
        break_target: usize,
        fallback_continue_target: usize,
    ) {
        for jump in loop_context.break_jumps {
            self.patch_jump(jump, break_target);
        }

        let continue_target = match loop_context.continue_target {
            Some(target) => target,
            None => fallback_continue_target,
        };

        for jump in loop_context.continue_jumps {
            self.patch_jump(jump, continue_target);
        }
    }
}

pub fn create_anonymous_bytecode_function(
    env: &mut PengEnv,
    context: PengGeneratorContext,
) -> PengValuePtr {
    let function = create_bytecode_function_value(
        context,
        0,
    );

    env.create_value(function)
}

fn create_bytecode_function_value(
    mut context: PengGeneratorContext,
    generics_count: usize,
) -> PengValue {
    context.push_const_and_const_instruction(PengValue::Nil);
    context.bytecode.push(PengInstruction::Return);

    PengValue::Function(
        PengFunction::Bytecode(
            PengBytecodeFunction {
                bytecode: context.bytecode,
                consts: context.consts,
                generics_count,
                using_values: Vec::new(),
            },
        ),
    )
}

pub fn generate_expression(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    expression: &PengPositionedExpression,
) -> Result<(), PengError> {
    match &expression.value {
        PengExpression::Literal(literal) => {
            generate_literal(env, context, literal)
        }
        PengExpression::Identifier(identifier) => {
            generate_identifier(env, context, identifier)
        }
        PengExpression::Unary {
            operator,
            value,
        } => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match operator {
                PengUnaryOperator::Negate => {
                    context.bytecode.push(PengInstruction::Negate);
                }
                PengUnaryOperator::Not => {
                    context.bytecode.push(PengInstruction::Not);
                }
            }

            Ok(())
        }
        PengExpression::Binary {
            left,
            operator,
            right,
        } => {
            match generate_expression(env, context, left) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            match generate_expression(env, context, right) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            generate_binary_operator(context, operator);
            Ok(())
        }
        PengExpression::Type(_) => {
            todo!("generate type value expression")
        }
        PengExpression::FuncCall(call) => {
            generate_function_call(env, context, call)
        }
        PengExpression::MethodCall(_) => {
            todo!("generate method call expression")
        }
        PengExpression::AttributeAccess(_) => {
            todo!("generate attribute access expression")
        }
        PengExpression::MemberAccess(_) => {
            todo!("generate member access expression")
        }
        PengExpression::Index(_) => {
            todo!("generate index expression")
        }
        PengExpression::ObjectConstruction(_) => {
            todo!("generate object construction expression")
        }
        PengExpression::OperationCall { .. } => {
            todo!("generate operation call expression")
        }
        PengExpression::Try { .. } => {
            todo!("generate try expression")
        }
    }
}

fn generate_literal(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    literal: &PengPositionedLiteral,
) -> Result<(), PengError> {
    let value = match &literal.value {
        PengLiteral::Nil => PengValue::Nil,
        PengLiteral::Int(value) => PengValue::Int(*value),
        PengLiteral::Uint(value) => PengValue::Uint(*value),
        PengLiteral::Byte(value) => PengValue::Byte(*value),
        PengLiteral::Float32(value) => PengValue::Float32(*value),
        PengLiteral::Float64(value) => PengValue::Float64(*value),
        PengLiteral::Bool(value) => PengValue::Bool(*value),
        PengLiteral::String(value) => PengValue::String(value.clone()),
        PengLiteral::Type(_) => {
            todo!("generate type literal")
        }
        PengLiteral::Function(function) => {
            match generate_function_value(
                env,
                &function.generics,
                &function.params,
                &function.body,
            ) {
                Ok(value) => value,
                Err(e) => return Err(e),
            }
        }
        PengLiteral::Module(_) => {
            todo!("generate module literal")
        }
        PengLiteral::Vector(_) => {
            todo!("generate vector literal")
        }
        PengLiteral::Operation(_) => {
            todo!("generate operation literal")
        }
        PengLiteral::Object(_) => {
            todo!("generate object literal")
        }
    };

    context.push_const_and_const_instruction(value);
    Ok(())
}

pub fn generate_function_declaration_value(
    env: &PengEnv,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<PengValue, PengError> {
    generate_function_value(
        env,
        &declaration.value.generics,
        &declaration.value.params,
        &declaration.value.body,
    )
}

fn generate_function_value(
    env: &PengEnv,
    generics: &Vec<PengPositioned<String>>,
    params: &Vec<PengPositionedFunctionParam>,
    body: &Vec<PengPositionedStatement>,
) -> Result<PengValue, PengError> {
    let mut context = PengGeneratorContext::new();

    for param in params {
        context.create_local(
            param.value.name.value.clone(),
        );
    }

    match generate_statements(env, &mut context, body) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(create_bytecode_function_value(
        context,
        generics.len(),
    ))
}

fn generate_function_call(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    call: &PengFuncCallExpression,
) -> Result<(), PengError> {
    match generate_expression(env, context, &call.function) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    for generic in &call.generics {
        match generate_expression(env, context, generic) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    for arg in &call.args {
        match generate_expression(env, context, arg) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    context.bytecode.push(PengInstruction::Call {
        generics: call.generics.len(),
        params: call.args.len(),
    });

    Ok(())
}

fn generate_identifier(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    identifier: &PengPositioned<String>,
) -> Result<(), PengError> {
    match context.get_local(&identifier.value) {
        Some(local) => {
            context.bytecode.push(
                PengInstruction::PushLocal(local),
            );
            return Ok(());
        }
        None => {}
    }

    match env.get_global(&identifier.value) {
        Some(value_ptr) => {
            context.bytecode.push(
                PengInstruction::PushValue(value_ptr),
            );
            Ok(())
        }
        None => Err(PengError::new_positioned_message(
            format!("unknown value '{}'", identifier.value),
            identifier.position.clone(),
        )),
    }
}

pub fn generate_statements(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    for statement in statements {
        match generate_statement(env, context, statement) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

fn generate_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengPositionedStatement,
) -> Result<(), PengError> {
    match &statement.value {
        PengStatement::Declaration(
            PengDeclaration::Variable(variable),
        ) => {
            generate_local_variable(env, context, variable)
        }
        PengStatement::Declaration(
            PengDeclaration::As(declaration),
        ) => {
            generate_local_as_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(
            PengDeclaration::Function(declaration),
        ) => {
            generate_local_function_declaration(
                env,
                context,
                declaration,
            )
        }
        PengStatement::Declaration(_) => {
            todo!("generate non-function local declaration")
        }
        PengStatement::Block(statements) => {
            context.push_scope();

            let result = generate_statements(
                env,
                context,
                statements,
            );

            context.pop_scope();

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        }
        PengStatement::Return(value) => {
            match value {
                Some(value) => {
                    match generate_expression(
                        env,
                        context,
                        value,
                    ) {
                        Ok(()) => {}
                        Err(e) => return Err(e),
                    }
                }
                None => {
                    context.push_const_and_const_instruction(PengValue::Nil);
                }
            }

            context.bytecode.push(PengInstruction::Return);
            Ok(())
        }
        PengStatement::Assign(assignment) => {
            generate_assignment(env, context, assignment)
        }
        PengStatement::Expression(expression) => {
            match generate_expression(env, context, expression) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            context.bytecode.push(PengInstruction::Pop(1));
            Ok(())
        }
        PengStatement::If(if_statement) => {
            generate_if_statement(
                env,
                context,
                if_statement,
            )
        }
        PengStatement::While(while_statement) => {
            generate_while_statement(
                env,
                context,
                while_statement,
            )
        }
        PengStatement::For(for_statement) => {
            generate_for_statement(
                env,
                context,
                for_statement,
            )
        }
        PengStatement::ForEach(_) => {
            todo!("generate foreach statement")
        }
        PengStatement::Loop(body) => {
            generate_loop_statement(env, context, body)
        }
        PengStatement::Break => {
            if context.emit_break() {
                Ok(())
            } else {
                Err(PengError::new_positioned_message(
                    "'break' used outside a loop".to_string(),
                    statement.position.clone(),
                ))
            }
        }
        PengStatement::Continue => {
            if context.emit_continue() {
                Ok(())
            } else {
                Err(PengError::new_positioned_message(
                    "'continue' used outside a loop".to_string(),
                    statement.position.clone(),
                ))
            }
        }
    }
}

fn generate_scoped_statements(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statements: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    context.push_scope();

    let result = generate_statements(
        env,
        context,
        statements,
    );

    context.pop_scope();

    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

fn generate_if_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengIfStatement,
) -> Result<(), PengError> {
    match generate_expression(
        env,
        context,
        &statement.condition,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let false_jump = context.emit_jump_if_false();

    match generate_scoped_statements(
        env,
        context,
        &statement.then_branch,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match &statement.else_branch {
        Some(else_branch) => {
            let end_jump = context.emit_jump();
            let else_start = context.bytecode.len();
            context.patch_jump(false_jump, else_start);

            match generate_scoped_statements(
                env,
                context,
                else_branch,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            let end = context.bytecode.len();
            context.patch_jump(end_jump, end);
        }
        None => {
            let end = context.bytecode.len();
            context.patch_jump(false_jump, end);
        }
    }

    Ok(())
}

fn generate_while_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengWhileStatement,
) -> Result<(), PengError> {
    let condition_start = context.bytecode.len();

    match generate_expression(
        env,
        context,
        &statement.condition,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let end_jump = context.emit_jump_if_false();
    context.push_loop(Some(condition_start));

    let body_result = generate_scoped_statements(
        env,
        context,
        &statement.body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            return Err(e);
        }
    }

    context.bytecode.push(
        PengInstruction::Jump(condition_start),
    );

    let end = context.bytecode.len();
    context.patch_jump(end_jump, end);

    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            return Err(PengError::new_positioned_message(
                "missing while loop generation context".to_string(),
                statement.condition.position.clone(),
            ));
        }
    };

    context.patch_loop(
        loop_context,
        end,
        condition_start,
    );

    Ok(())
}

fn generate_loop_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    body: &Vec<PengPositionedStatement>,
) -> Result<(), PengError> {
    let loop_start = context.bytecode.len();
    context.push_loop(Some(loop_start));

    let body_result = generate_scoped_statements(
        env,
        context,
        body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            return Err(e);
        }
    }

    context.bytecode.push(
        PengInstruction::Jump(loop_start),
    );

    let end = context.bytecode.len();
    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            return Err(PengError::new_message(
                "missing loop generation context".to_string(),
            ));
        }
    };

    context.patch_loop(loop_context, end, loop_start);
    Ok(())
}

fn generate_for_statement(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    statement: &PengForStatement,
) -> Result<(), PengError> {
    context.push_scope();

    match &statement.initializer {
        Some(initializer) => {
            match generate_statement(
                env,
                context,
                initializer,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_scope();
                    return Err(e);
                }
            }
        }
        None => {}
    }

    let condition_start = context.bytecode.len();
    let end_jump = match &statement.condition {
        Some(condition) => {
            match generate_expression(
                env,
                context,
                condition,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_scope();
                    return Err(e);
                }
            }

            Some(context.emit_jump_if_false())
        }
        None => None,
    };

    context.push_loop(None);

    let body_result = generate_scoped_statements(
        env,
        context,
        &statement.body,
    );

    match body_result {
        Ok(()) => {}
        Err(e) => {
            context.pop_loop();
            context.pop_scope();
            return Err(e);
        }
    }

    let increment_start = context.bytecode.len();

    match &statement.increment {
        Some(increment) => {
            match generate_statement(
                env,
                context,
                increment,
            ) {
                Ok(()) => {}
                Err(e) => {
                    context.pop_loop();
                    context.pop_scope();
                    return Err(e);
                }
            }
        }
        None => {}
    }

    context.bytecode.push(
        PengInstruction::Jump(condition_start),
    );

    let end = context.bytecode.len();

    match end_jump {
        Some(end_jump) => {
            context.patch_jump(end_jump, end);
        }
        None => {}
    }

    let loop_context = match context.pop_loop() {
        Some(loop_context) => loop_context,
        None => {
            context.pop_scope();
            return Err(PengError::new_message(
                "missing for loop generation context".to_string(),
            ));
        }
    };

    context.patch_loop(
        loop_context,
        end,
        increment_start,
    );
    context.pop_scope();

    Ok(())
}

fn generate_local_variable(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    variable: &PengPositionedVariableDeclaration,
) -> Result<(), PengError> {
    match &variable.value.value {
        Some(value) => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
        None => {
            context.push_const_and_const_instruction(PengValue::Nil);
        }
    }

    let local = context.create_local(
        variable.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}

fn generate_local_as_declaration(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
) -> Result<(), PengError> {
    match generate_expression(
        env,
        context,
        &declaration.value.value,
    ) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let local = context.create_local(
        declaration.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}

fn generate_local_function_declaration(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    declaration: &PengPositionedFunctionDeclaration,
) -> Result<(), PengError> {
    let value = match generate_function_declaration_value(
        env,
        declaration,
    ) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    context.push_const_and_const_instruction(value);

    let local = context.create_local(
        declaration.value.name.value.clone(),
    );
    context.bytecode.push(
        PengInstruction::StoreLocal(local),
    );

    Ok(())
}

fn generate_assignment(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    assignment: &PengAssignStatement,
) -> Result<(), PengError> {
    match assignment {
        PengAssignStatement::Assign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                None,
            )
        }
        PengAssignStatement::AddAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Add),
            )
        }
        PengAssignStatement::SubtractAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Subtract),
            )
        }
        PengAssignStatement::MultiplyAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Multiply),
            )
        }
        PengAssignStatement::DivideAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Divide),
            )
        }
        PengAssignStatement::PowerAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Power),
            )
        }
        PengAssignStatement::RemainderAssign { target, value } => {
            generate_assignment_value(
                env,
                context,
                target,
                value,
                Some(PengInstruction::Remainder),
            )
        }
    }
}

fn generate_assignment_value(
    env: &PengEnv,
    context: &mut PengGeneratorContext,
    target: &PengPositionedExpression,
    value: &PengPositionedExpression,
    operation: Option<PengInstruction>,
) -> Result<(), PengError> {
    let identifier = match &target.value {
        PengExpression::Identifier(identifier) => identifier,
        PengExpression::AttributeAccess(_) => {
            todo!("generate attribute assignment")
        }
        PengExpression::Index(_) => {
            todo!("generate index assignment")
        }
        _ => {
            return Err(PengError::new_positioned_message(
                "unsupported assignment target".to_string(),
                target.position.clone(),
            ));
        }
    };

    let local = context.get_local(&identifier.value);
    let global = if local.is_none() {
        env.get_global(&identifier.value)
    } else {
        None
    };

    match global {
        Some(value_ptr) => {
            context.bytecode.push(
                PengInstruction::PushValueRef(value_ptr),
            );
        }
        None => {}
    }

    match operation {
        Some(operation) => {
            match local {
                Some(local) => {
                    context.bytecode.push(
                        PengInstruction::PushLocal(local),
                    );
                }
                None => match global {
                    Some(value_ptr) => {
                        context.bytecode.push(
                            PengInstruction::PushValue(value_ptr),
                        );
                    }
                    None => {
                        return Err(PengError::new_positioned_message(
                            format!(
                                "unknown assignment target '{}'",
                                identifier.value,
                            ),
                            identifier.position.clone(),
                        ));
                    }
                },
            }

            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            context.bytecode.push(operation);
        }
        None => {
            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
    }

    match local {
        Some(local) => {
            context.bytecode.push(
                PengInstruction::StoreLocal(local),
            );
            Ok(())
        }
        None => match global {
            Some(_) => {
                context.bytecode.push(
                    PengInstruction::StoreValue,
                );
                Ok(())
            }
            None => Err(PengError::new_positioned_message(
                format!(
                    "unknown assignment target '{}'",
                    identifier.value,
                ),
                identifier.position.clone(),
            )),
        },
    }
}

fn generate_binary_operator(
    context: &mut PengGeneratorContext,
    operator: &PengBinaryOperator,
) {
    let instruction = match operator {
        PengBinaryOperator::Add => PengInstruction::Add,
        PengBinaryOperator::Subtract => PengInstruction::Subtract,
        PengBinaryOperator::Multiply => PengInstruction::Multiply,
        PengBinaryOperator::Divide => PengInstruction::Divide,
        PengBinaryOperator::Power => PengInstruction::Power,
        PengBinaryOperator::Remainder => PengInstruction::Remainder,
        PengBinaryOperator::Concat => PengInstruction::Concat,
        PengBinaryOperator::And => PengInstruction::And,
        PengBinaryOperator::Or => PengInstruction::Or,
        PengBinaryOperator::Equals => PengInstruction::Equals,
        PengBinaryOperator::NotEquals => PengInstruction::NotEquals,
        PengBinaryOperator::GreaterThan => PengInstruction::GreaterThan,
        PengBinaryOperator::GreaterEqualsThan => {
            PengInstruction::GreaterEqualsThan
        }
        PengBinaryOperator::LessThan => PengInstruction::LessThan,
        PengBinaryOperator::LessEqualsThan => {
            PengInstruction::LessEqualsThan
        }
        PengBinaryOperator::As => {
            todo!("generate type conversion")
        }
    };

    context.bytecode.push(instruction);
}
