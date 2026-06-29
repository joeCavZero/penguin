use std::collections::HashMap;

use crate::core::*;
pub use crate::generator::generate::*;

use crate::parser::*;

pub struct PengGeneratorContext {
    pub bytecode: Vec<PengInstruction>,
    pub positions: Vec<PengPosition>,
    pub consts: Vec<PengValue>,

    globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
    using_globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,

    scopes: Vec<HashMap<String, usize>>,
    next_local: usize,
    loops: Vec<PengLoopContext>,
}
pub struct PengLoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
    continue_target: Option<usize>,
}

impl PengGeneratorContext {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            positions: Vec::new(),
            consts: Vec::new(),

            globals: HashMap::new(),
            using_globals: HashMap::new(),

            scopes: vec![HashMap::new()],
            next_local: 0,
            loops: Vec::new(),
        }
    }

    pub fn new_child_context(&self) -> Self {
        Self {
            bytecode: Vec::new(),
            positions: Vec::new(),
            consts: Vec::new(),

            globals: self.globals.clone(),
            using_globals: self.using_globals.clone(),

            scopes: vec![HashMap::new()],
            next_local: 0,
            loops: Vec::new(),
        }
    }

    pub fn push_positioned_instruction(&mut self, instr: PengInstruction, pos: PengPosition) {
        debug_assert_eq!(self.bytecode.len(), self.positions.len());
        self.bytecode.push(instr);
        self.positions.push(pos);
        debug_assert_eq!(self.bytecode.len(), self.positions.len());
    }

    pub fn into_globals(self) -> HashMap<PengNamePoolPtr, PengBindedHeapPtr> {
        self.globals
    }

    pub fn globals(&self) -> &HashMap<PengNamePoolPtr, PengBindedHeapPtr> {
        &self.globals
    }

    pub fn insert_global(&mut self, name: PengNamePoolPtr, value: PengBindedHeapPtr) {
        self.globals.insert(name, value);
    }

    pub fn use_global(&mut self, name: PengNamePoolPtr, value: PengBindedHeapPtr) {
        self.using_globals.insert(name, value);
    }

    pub fn use_globals(
        &mut self,
        env: &mut PengEnv,
        globals: HashMap<PengNamePoolPtr, PengBindedHeapPtr>,
    ) {
        for (name, value) in globals {
            self.using_globals.insert(name, value.clone());
            env.pinned_mut().insert(*value.value());
        }
    }

    pub fn get_global(&self, name: PengNamePoolPtr) -> Option<&PengBindedHeapPtr> {
        match self.globals.get(&name) {
            Some(value) => Some(value),
            None => self.using_globals.get(&name),
        }
    }

    pub fn get_own_global(&self, name: PengNamePoolPtr) -> Option<&PengBindedHeapPtr> {
        self.globals.get(&name)
    }

    pub fn push_const_and_const_instruction(
        &mut self,
        env: &mut PengEnv,
        value: PengValue,
        pos: PengPosition,
    ) {
        if let PengValue::Box(PengBox::String(s)) = value {
            let name_ptr = env.ensure_pooled_name_ptr(s);
            self.push_positioned_instruction(PengInstruction::PushString(name_ptr), pos);
            return;
        }

        let mut const_index = None;

        for index in 0..self.consts.len() {
            let existing = match self.consts.get(index) {
                Some(existing) => existing,
                None => break,
            };

            if existing.equals(&value) {
                const_index = Some(index);
                break;
            }
        }

        let const_index = match const_index {
            Some(const_index) => const_index,
            None => {
                let const_index = self.consts.len();
                self.consts.push(value);
                const_index
            }
        };

        self.push_positioned_instruction(PengInstruction::PushConst(const_index), pos);
    }

    pub fn create_local(&mut self, name: String) -> usize {
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

    pub fn create_temporary_local(&mut self) -> usize {
        let local = self.next_local;
        self.next_local += 1;
        local
    }

    pub fn insert_local(&mut self, name: String, local: usize) {
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
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn get_local(&self, name: &str) -> Option<usize> {
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

    pub fn emit_jump(&mut self, pos: PengPosition) -> usize {
        let instruction_index = self.bytecode.len();
        self.push_positioned_instruction(PengInstruction::Jump(0), pos);
        instruction_index
    }

    pub fn emit_jump_if_false(&mut self, pos: PengPosition) -> usize {
        let instruction_index = self.bytecode.len();
        self.push_positioned_instruction(PengInstruction::JumpIfFalse(0), pos);
        instruction_index
    }

    pub fn patch_jump(&mut self, instruction_index: usize, target: usize) {
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

    pub fn push_loop(&mut self, continue_target: Option<usize>) {
        self.loops.push(PengLoopContext {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            continue_target,
        });
    }

    pub fn pop_loop(&mut self) -> Option<PengLoopContext> {
        self.loops.pop()
    }

    pub fn emit_break(&mut self, pos: PengPosition) -> bool {
        if self.loops.is_empty() {
            return false;
        }

        let jump = self.emit_jump(pos);

        match self.loops.last_mut() {
            Some(loop_context) => {
                loop_context.break_jumps.push(jump);
                true
            }
            None => false,
        }
    }

    pub fn emit_continue(&mut self, pos: PengPosition) -> bool {
        if self.loops.is_empty() {
            return false;
        }

        let jump = self.emit_jump(pos);

        match self.loops.last_mut() {
            Some(loop_context) => {
                loop_context.continue_jumps.push(jump);
                true
            }
            None => false,
        }
    }

    pub fn patch_loop(
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

pub fn generate_identifier(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    identifier: &PengPositioned<String>,
) -> Result<(), PengError> {
    match context.get_local(&identifier.value) {
        Some(local) => {
            context.push_positioned_instruction(
                PengInstruction::PushLocal(local),
                identifier.position.clone(),
            );
            return Ok(());
        }

        None => {}
    }

    let name_ptr = env.ensure_pooled_name_ptr(identifier.value.clone());

    match context.get_global(name_ptr).cloned() {
        Some(value) => {
            context.push_positioned_instruction(
                PengInstruction::PushHeap(*value.value()),
                identifier.position.clone(),
            );

            match &value {
                PengBinded::Immutable(_) => {
                    context.push_positioned_instruction(
                        PengInstruction::MakeImmutable,
                        identifier.position.clone(),
                    );
                }

                PengBinded::Mutable(_) => {}
            }

            Ok(())
        }

        None => Err(PengError::new_positioned_message(
            format!("unknown value '{}'", identifier.value),
            identifier.position.clone(),
        )),
    }
}

pub fn declaration_position(declaration: &PengDeclaration) -> PengPosition {
    match declaration {
        PengDeclaration::Var(declaration) => declaration.position.clone(),
        PengDeclaration::As(declaration) => declaration.position.clone(),
        PengDeclaration::Function(declaration) => declaration.position.clone(),
        PengDeclaration::Type(declaration) => declaration.position.clone(),
        PengDeclaration::Module(declaration) => declaration.position.clone(),
        PengDeclaration::Operation(declaration) => declaration.position.clone(),
    }
}

pub fn generate_local_variable(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    variable: &PengPositionedVariableDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    match &variable.value.value {
        Some(value) => match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        },

        None => {
            if immutable {
                return Err(PengError::new_positioned_message(
                    "const declaration must have an initializer".to_string(),
                    variable.position.clone(),
                ));
            }

            context.push_const_and_const_instruction(
                env,
                PengValue::Cell(PengCell::Nil),
                variable.position.clone(),
            );
        }
    }

    generate_make_immutable_if_needed(context, immutable, variable.position.clone());

    let local = context.create_local(variable.value.name.value.clone());
    context.push_positioned_instruction(
        PengInstruction::StoreLocal(local),
        variable.position.clone(),
    );

    Ok(())
}

pub fn generate_local_as_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedAsDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    match generate_expression(env, context, &declaration.value.value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }
    generate_make_immutable_if_needed(context, immutable, declaration.position.clone());

    let local = context.create_local(declaration.value.name.value.clone());
    context.push_positioned_instruction(
        PengInstruction::StoreLocal(local),
        declaration.position.clone(),
    );
    Ok(())
}

pub fn generate_local_type_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    let value = match &declaration.value.value {
        Some(value) => value,
        None => {
            return generate_structured_local_type_declaration(
                env,
                context,
                declaration,
                immutable,
            );
        }
    };

    match generate_type_expression(env, context, value) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }

    let local = context.create_local(declaration.value.name.value.clone());
    context.push_positioned_instruction(
        PengInstruction::StoreLocal(local),
        declaration.position.clone(),
    );

    Ok(())
}

pub fn generate_structured_local_type_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedTypeDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    let literal = PengTypeLiteral {
        supers: declaration.value.supers.clone(),
        fields: declaration.value.fields.clone(),
        functions: declaration.value.functions.clone(),
    };

    match generate_type_literal(env, context, &literal, declaration.position.clone()) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    };
    generate_make_immutable_if_needed(context, immutable, declaration.position.clone());

    let local = context.create_local(declaration.value.name.value.clone());
    context.push_positioned_instruction(
        PengInstruction::StoreLocal(local),
        declaration.position.clone(),
    );
    Ok(())
}

pub fn generate_structured_global_type(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    type_ptr: PengHeapPtr,
    declaration: &PengPositionedTypeDeclaration,
) -> Result<(), PengError> {
    context.push_positioned_instruction(
        PengInstruction::PushHeapRef(type_ptr),
        declaration.position.clone(),
    );

    for super_type in &declaration.value.supers {
        match generate_expression(env, context, super_type) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating structured global type super".to_string(),
                )));
            }
        }
    }

    context.push_positioned_instruction(
        PengInstruction::CreateSuperType(declaration.value.supers.len()),
        declaration.position.clone(),
    );

    context.push_positioned_instruction(PengInstruction::StoreHeap, declaration.position.clone());

    for binded_field in &declaration.value.fields {
        let (field, immutable) = match binded_field {
            PengBinded::Mutable(field) => (field, false),
            PengBinded::Immutable(field) => (field, true),
        };
        context.push_positioned_instruction(
            PengInstruction::PushHeap(type_ptr),
            field.position.clone(),
        );

        let name = env.ensure_pooled_name_ptr(field.value.name.value.clone());

        match &field.value.value {
            Some(value) => match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating structured global type field".to_string(),
                    )));
                }
            },

            None => {
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Cell(PengCell::Nil),
                    field.position.clone(),
                );
            }
        }

        generate_make_immutable_if_needed(context, immutable, field.position.clone());
        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            field.position.clone(),
        );
    }

    for binded_function in &declaration.value.functions {
        let (function, immutable) = match binded_function {
            PengBinded::Mutable(function) => (function, false),
            PengBinded::Immutable(function) => (function, true),
        };
        context.push_positioned_instruction(
            PengInstruction::PushHeap(type_ptr),
            function.position.clone(),
        );

        let name = env.ensure_pooled_name_ptr(function.value.name.value.clone());

        let value = match generate_function_declaration_value(env, context, function) {
            Ok(value) => value,
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating structured global type function".to_string(),
                )));
            }
        };

        context.push_const_and_const_instruction(
            env,
            PengValue::Box(value),
            function.position.clone(),
        );

        generate_make_immutable_if_needed(context, immutable, function.position.clone());
        context.push_positioned_instruction(
            PengInstruction::SetAttribute(name),
            function.position.clone(),
        );
    }

    Ok(())
}

pub fn generate_assignment(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    assignment: &PengAssignStatement,
) -> Result<(), PengError> {
    match assignment {
        PengAssignStatement::Assign { target, value } => {
            generate_assignment_value(env, context, target, value, None)
        }
        PengAssignStatement::AddAssign { target, value } => {
            generate_assignment_value(env, context, target, value, Some(PengInstruction::Add))
        }
        PengAssignStatement::SubtractAssign { target, value } => {
            generate_assignment_value(env, context, target, value, Some(PengInstruction::Subtract))
        }
        PengAssignStatement::MultiplyAssign { target, value } => {
            generate_assignment_value(env, context, target, value, Some(PengInstruction::Multiply))
        }
        PengAssignStatement::DivideAssign { target, value } => {
            generate_assignment_value(env, context, target, value, Some(PengInstruction::Divide))
        }
        PengAssignStatement::PowerAssign { target, value } => {
            generate_assignment_value(env, context, target, value, Some(PengInstruction::Power))
        }
        PengAssignStatement::RemainderAssign { target, value } => generate_assignment_value(
            env,
            context,
            target,
            value,
            Some(PengInstruction::Remainder),
        ),
    }
}

pub fn generate_assignment_value(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    target: &PengPositionedExpression,
    value: &PengPositionedExpression,
    operation: Option<PengInstruction>,
) -> Result<(), PengError> {
    let identifier = match &target.value {
        PengExpression::Identifier(identifier) => identifier,
        PengExpression::AttributeAccess(attribute) => {
            return generate_attribute_assignment(
                env,
                context,
                attribute,
                value,
                operation,
                target.position.clone(),
            );
        }
        PengExpression::Index(index) => {
            return generate_index_assignment(
                env,
                context,
                index,
                value,
                operation,
                target.position.clone(),
            );
        }
        PengExpression::MemberAccess(member) => {
            return generate_member_assignment(
                env,
                context,
                member,
                value,
                operation,
                target.position.clone(),
            );
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
        get_allocated_global(env, context, &identifier.value)
    } else {
        None
    };

    match &global {
        Some(PengBinded::Mutable(value_ptr)) => {
            context.push_positioned_instruction(
                PengInstruction::PushHeapRef(*value_ptr),
                target.position.clone(),
            );
        }
        Some(PengBinded::Immutable(_)) => {
            return Err(PengError::CannotMutateImmutable);
        }
        None => {}
    }

    match operation {
        Some(operation) => {
            match local {
                Some(local) => {
                    context.push_positioned_instruction(
                        PengInstruction::PushLocal(local),
                        target.position.clone(),
                    );
                }
                None => match &global {
                    Some(PengBinded::Mutable(value_ptr)) => {
                        context.push_positioned_instruction(
                            PengInstruction::PushHeap(*value_ptr),
                            target.position.clone(),
                        );
                    }
                    Some(PengBinded::Immutable(_)) => {
                        return Err(PengError::CannotMutateImmutable);
                    }
                    None => {
                        return Err(PengError::new_positioned_message(
                            format!("unknown assignment target '{}'", identifier.value,),
                            identifier.position.clone(),
                        ));
                    }
                },
            }

            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generator_utils".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(operation, target.position.clone());
        }
        None => match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        },
    }

    match local {
        Some(local) => {
            context.push_positioned_instruction(
                PengInstruction::StoreLocal(local),
                target.position.clone(),
            );
            Ok(())
        }
        None => match &global {
            Some(_) => {
                context.push_positioned_instruction(
                    PengInstruction::StoreHeap,
                    target.position.clone(),
                );
                Ok(())
            }
            None => Err(PengError::new_positioned_message(
                format!("unknown assignment target '{}'", identifier.value,),
                identifier.position.clone(),
            )),
        },
    }
}

pub fn generate_index_assignment(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    index: &PengIndexExpression,
    value: &PengPositionedExpression,
    operation: Option<PengInstruction>,
    pos: PengPosition,
) -> Result<(), PengError> {
    if operation.is_none() {
        match generate_expression(env, context, &index.object) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        match generate_expression(env, context, &index.index) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        context.push_positioned_instruction(PengInstruction::SetIndex, pos);
        return Ok(());
    }

    let object_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &index.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreLocal(object_local), pos.clone());

    let index_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &index.index) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreLocal(index_local), pos.clone());
    context.push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
    context.push_positioned_instruction(PengInstruction::PushLocal(index_local), pos.clone());

    match operation {
        Some(operation) => {
            context
                .push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
            context
                .push_positioned_instruction(PengInstruction::PushLocal(index_local), pos.clone());
            context.push_positioned_instruction(PengInstruction::GetIndex, pos.clone());

            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generator_utils".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(operation, pos.clone());
        }
        None => unreachable!(),
    }

    context.push_positioned_instruction(PengInstruction::SetIndex, pos);
    Ok(())
}

pub fn generate_attribute_assignment(
    env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    attribute: &PengAttributeAccessExpression,
    value: &PengPositionedExpression,
    operation: Option<PengInstruction>,
    pos: PengPosition,
) -> Result<(), PengError> {
    let name = env.ensure_pooled_name_ptr(attribute.name.value.clone());

    if operation.is_none() {
        match generate_expression(env, context, &attribute.object) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        context.push_positioned_instruction(PengInstruction::SetAttribute(name), pos);
        return Ok(());
    }

    let object_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &attribute.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreLocal(object_local), pos.clone());
    context.push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());

    match operation {
        Some(operation) => {
            context
                .push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
            context.push_positioned_instruction(PengInstruction::GetAttribute(name), pos.clone());

            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generator_utils".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(operation, pos.clone());
        }
        None => unreachable!(),
    }

    context.push_positioned_instruction(PengInstruction::SetAttribute(name), pos);
    Ok(())
}

pub fn generate_binary_operator(
    context: &mut PengGeneratorContext,
    operator: &PengBinaryOperator,
    pos: PengPosition,
) {
    let instruction = match operator {
        PengBinaryOperator::Add => PengInstruction::Add,
        PengBinaryOperator::Subtract => PengInstruction::Subtract,
        PengBinaryOperator::Multiply => PengInstruction::Multiply,
        PengBinaryOperator::Divide => PengInstruction::Divide,
        PengBinaryOperator::Power => PengInstruction::Power,
        PengBinaryOperator::Remainder => PengInstruction::Remainder,
        PengBinaryOperator::Concat => PengInstruction::Concat,
        PengBinaryOperator::NonShortCircuitAnd => PengInstruction::And,
        PengBinaryOperator::NonShortCircuitOr => PengInstruction::Or,
        PengBinaryOperator::ShortCircuitAnd | PengBinaryOperator::ShortCircuitOr => {
            unreachable!("short-circuit operators are generated separately")
        }
        PengBinaryOperator::Equals => PengInstruction::Equals,
        PengBinaryOperator::NotEquals => PengInstruction::NotEquals,
        PengBinaryOperator::GreaterThan => PengInstruction::GreaterThan,
        PengBinaryOperator::GreaterEqualsThan => PengInstruction::GreaterEqualsThan,
        PengBinaryOperator::LessThan => PengInstruction::LessThan,
        PengBinaryOperator::LessEqualsThan => PengInstruction::LessEqualsThan,
        PengBinaryOperator::As => PengInstruction::Convert,
    };

    context.push_positioned_instruction(instruction, pos);
}

pub fn generate_member_assignment(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    member: &PengMemberAccessExpression,
    value: &PengPositionedExpression,
    operation: Option<PengInstruction>,
    pos: PengPosition,
) -> Result<(), PengError> {
    let name = env.ensure_pooled_name_ptr(member.name.value.clone());

    if operation.is_none() {
        match generate_expression(env, context, &member.object) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        match generate_expression(env, context, value) {
            Ok(()) => {}
            Err(e) => {
                return Err(e.push(PengError::InvalidState(
                    "failed while generating generator_utils".to_string(),
                )));
            }
        }

        context.push_positioned_instruction(PengInstruction::SetMember(name), pos);
        return Ok(());
    }

    let object_local = generate_reserved_temporary_local(env, context, pos.clone());

    match generate_expression(env, context, &member.object) {
        Ok(()) => {}
        Err(e) => {
            return Err(e.push(PengError::InvalidState(
                "failed while generating generator_utils".to_string(),
            )));
        }
    }

    context.push_positioned_instruction(PengInstruction::StoreLocal(object_local), pos.clone());
    context.push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());

    match operation {
        Some(operation) => {
            context
                .push_positioned_instruction(PengInstruction::PushLocal(object_local), pos.clone());
            context.push_positioned_instruction(PengInstruction::GetMember(name), pos.clone());

            match generate_expression(env, context, value) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e.push(PengError::InvalidState(
                        "failed while generating generator_utils".to_string(),
                    )));
                }
            }

            context.push_positioned_instruction(operation, pos.clone());
        }
        None => unreachable!(),
    }

    context.push_positioned_instruction(PengInstruction::SetMember(name), pos);
    Ok(())
}

pub fn generate_local_module_declaration(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedModuleDeclaration,
    immutable: bool,
) -> Result<(), PengError> {
    context.push_positioned_instruction(
        PengInstruction::CreateEmptyModule,
        declaration.position.clone(),
    );

    for binded_declaration in &declaration.value.body {
        let declaration_value = match binded_declaration {
            PengBinded::Mutable(declaration) | PengBinded::Immutable(declaration) => declaration,
        };

        let name = match declaration_value {
            PengDeclaration::Var(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
        };

        let name_ptr = env.ensure_pooled_name_ptr(name);

        let member_pos = declaration_position(declaration_value);
        context.push_positioned_instruction(PengInstruction::Duplicate, member_pos.clone());

        match declaration_value {
            PengDeclaration::Var(declaration) => match &declaration.value.value {
                Some(value) => {
                    match generate_expression(env, context, value) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generator_utils".to_string(),
                            )));
                        }
                    };
                }
                None => {
                    context.push_const_and_const_instruction(
                        env,
                        PengValue::Cell(PengCell::Nil),
                        member_pos.clone(),
                    );
                }
            },

            PengDeclaration::As(declaration) => {
                match generate_expression(env, context, &declaration.value.value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
            }

            PengDeclaration::Function(declaration) => {
                let value = match generate_function_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }

            PengDeclaration::Operation(declaration) => {
                let value = match generate_operation_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }

            PengDeclaration::Type(declaration) => match &declaration.value.value {
                Some(value) => {
                    match generate_type_expression(env, context, value) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generator_utils".to_string(),
                            )));
                        }
                    };
                }
                None => {
                    let literal = PengTypeLiteral {
                        supers: declaration.value.supers.clone(),
                        fields: declaration.value.fields.clone(),
                        functions: declaration.value.functions.clone(),
                    };

                    match generate_type_literal(env, context, &literal, member_pos.clone()) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generator_utils".to_string(),
                            )));
                        }
                    };
                }
            },

            PengDeclaration::Module(declaration) => {
                match generate_module_declaration_value(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
            }
        }

        context.push_positioned_instruction(PengInstruction::SetMember(name_ptr), member_pos);
    }
    generate_make_immutable_if_needed(context, immutable, declaration.position.clone());
    let local = context.create_local(declaration.value.name.value.clone());
    context.push_positioned_instruction(
        PengInstruction::StoreLocal(local),
        declaration.position.clone(),
    );

    Ok(())
}

pub fn generate_module_declaration_value(
    env: &mut PengEnv,

    context: &mut PengGeneratorContext,
    declaration: &PengPositionedModuleDeclaration,
) -> Result<(), PengError> {
    context.push_positioned_instruction(
        PengInstruction::CreateEmptyModule,
        declaration.position.clone(),
    );

    for binded_declaration in &declaration.value.body {
        let declaration_value = match binded_declaration {
            PengBinded::Mutable(declaration) | PengBinded::Immutable(declaration) => declaration,
        };

        let name = match declaration_value {
            PengDeclaration::Var(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::As(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Function(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Type(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Module(declaration) => declaration.value.name.value.clone(),
            PengDeclaration::Operation(declaration) => declaration.value.name.value.clone(),
        };

        let name_ptr = env.ensure_pooled_name_ptr(name);

        let member_pos = declaration_position(declaration_value);
        context.push_positioned_instruction(PengInstruction::Duplicate, member_pos.clone());

        match declaration_value {
            PengDeclaration::Var(declaration) => match &declaration.value.value {
                Some(value) => match generate_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                },
                None => context.push_const_and_const_instruction(
                    env,
                    PengValue::Cell(PengCell::Nil),
                    member_pos.clone(),
                ),
            },

            PengDeclaration::As(declaration) => {
                match generate_expression(env, context, &declaration.value.value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
            }

            PengDeclaration::Function(declaration) => {
                let value = match generate_function_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }

            PengDeclaration::Operation(declaration) => {
                let value = match generate_operation_declaration_value(env, context, declaration) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
                context.push_const_and_const_instruction(
                    env,
                    PengValue::Box(value),
                    member_pos.clone(),
                );
            }

            PengDeclaration::Type(declaration) => match &declaration.value.value {
                Some(value) => match generate_type_expression(env, context, value) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                },
                None => {
                    let literal = PengTypeLiteral {
                        supers: declaration.value.supers.clone(),
                        fields: declaration.value.fields.clone(),
                        functions: declaration.value.functions.clone(),
                    };

                    match generate_type_literal(env, context, &literal, member_pos.clone()) {
                        Ok(()) => {}
                        Err(e) => {
                            return Err(e.push(PengError::InvalidState(
                                "failed while generating generator_utils".to_string(),
                            )));
                        }
                    };
                }
            },

            PengDeclaration::Module(declaration) => {
                match generate_module_declaration_value(env, context, declaration) {
                    Ok(()) => {}
                    Err(e) => {
                        return Err(e.push(PengError::InvalidState(
                            "failed while generating generator_utils".to_string(),
                        )));
                    }
                };
            }
        }

        context.push_positioned_instruction(PengInstruction::SetMember(name_ptr), member_pos);
    }

    Ok(())
}

pub fn generate_reserved_temporary_local(
    _env: &mut PengEnv,
    context: &mut PengGeneratorContext,
    pos: PengPosition,
) -> usize {
    let local = context.create_temporary_local();

    context.push_positioned_instruction(PengInstruction::ReserveLocal(local), pos);

    local
}
