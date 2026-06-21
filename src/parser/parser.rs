use std::iter::Peekable;
use std::slice::Iter;
use crate::core::*;
use crate::lexer::*;
use crate::parser::*;

pub type PengPeekablePositionedToken<'a> = Peekable<Iter<'a, PengPositionedToken>>;

pub type PengPositionedAST = PengPositioned<PengAST>;
pub type PengPositionedStatement = PengPositioned<PengStatement>;
pub type PengPositionedExpression = PengPositioned<PengExpression>;
pub type PengPositionedTypeExpression = PengPositioned<PengTypeExpression>;

pub type PengPositionedVariableDeclaration = PengPositioned<PengVariableDeclaration>;
pub type PengPositionedAsDeclaration = PengPositioned<PengAsDeclaration>;
pub type PengPositionedFunctionDeclaration = PengPositioned<PengFunctionDeclaration>;
pub type PengPositionedTypeDeclaration = PengPositioned<PengTypeDeclaration>;
pub type PengPositionedModuleDeclaration = PengPositioned<PengModuleDeclaration>;
pub type PengPositionedOperationDeclaration = PengPositioned<PengOperationDeclaration>;

pub type PengPositionedFunctionParam = PengPositioned<PengFunctionParam>;

pub type PengPositionedLiteral = PengPositioned<PengLiteral>;


#[derive(Debug, Clone)]
pub enum PengAST {
    Program(Vec<PengDeclaration>),
    Script(Vec<PengPositionedStatement>),
}

impl PengAST {
    pub fn pretty_print(&self) {
        println!("PengAST");

        match self {
            PengAST::Program(declarations) => {
                println!("  Program");

                for declaration in declarations {
                    Self::print_declaration(declaration, 2);
                }
            }
            PengAST::Script(statements) => {
                println!("  Script");

                for statement in statements {
                    Self::print_positioned_statement(statement, 2);
                }
            }
        }
    }

    fn print_indent(level: usize) {
        print!("{}", "  ".repeat(level));
    }

    fn print_position(position: &PengPosition) {
        match position {
            PengPosition::File {
                file_id,
                line,
                column,
            } => {
                print!(" [file:{}, line:{}]", file_id, line);
                match column {
                    Some(c) => print!(", column:{}", c),
                    None => print!(", column:?"),
                }
            }
            PengPosition::Source { line, column } => {
                print!(" @ line:{}", line);
                match column {
                    Some(c) => print!(", column:{}", c),
                    None => print!(", column:?"),
                }
            }
        }
    }

    fn print_positioned_statement(stmt: &PengPositionedStatement, level: usize) {
        Self::print_indent(level);
        print!("Statement");
        Self::print_position(&stmt.position);
        println!();

        Self::print_statement(&stmt.value, level + 1);
    }

    fn print_statement(stmt: &PengStatement, level: usize) {
        match stmt {
            PengStatement::Block(stmts) => {
                Self::print_indent(level);
                println!("Block");
                for stmt in stmts {
                    Self::print_positioned_statement(stmt, level + 1);
                }
            }

            PengStatement::Declaration(decl) => {
                Self::print_indent(level);
                println!("Declaration");
                Self::print_declaration(decl, level + 1);
            }

            PengStatement::Return(expr) => {
                Self::print_indent(level);
                println!("Return");
                match expr {
                    Some(expr) => Self::print_positioned_expression(expr, level + 1),
                    None => {
                        Self::print_indent(level + 1);
                        println!("None");
                    }
                }
            }

            PengStatement::If(if_stmt) => {
                Self::print_indent(level);
                println!("If");

                Self::print_indent(level + 1);
                println!("Condition");
                Self::print_positioned_expression(&if_stmt.condition, level + 2);

                Self::print_indent(level + 1);
                println!("Then");
                for stmt in &if_stmt.then_branch {
                    Self::print_positioned_statement(stmt, level + 2);
                }

                Self::print_indent(level + 1);
                println!("Else");
                match &if_stmt.else_branch {
                    Some(stmts) => {
                        for stmt in stmts {
                            Self::print_positioned_statement(stmt, level + 2);
                        }
                    }
                    None => {
                        Self::print_indent(level + 2);
                        println!("None");
                    }
                }
            }

            PengStatement::While(while_stmt) => {
                Self::print_indent(level);
                println!("While");

                Self::print_indent(level + 1);
                println!("Condition");
                Self::print_positioned_expression(&while_stmt.condition, level + 2);

                Self::print_indent(level + 1);
                println!("Body");
                for stmt in &while_stmt.body {
                    Self::print_positioned_statement(stmt, level + 2);
                }
            }

            PengStatement::For(for_stmt) => {
                Self::print_indent(level);
                println!("For");

                Self::print_indent(level + 1);
                println!("Initializer");
                match &for_stmt.initializer {
                    Some(stmt) => Self::print_positioned_statement(stmt, level + 2),
                    None => {
                        Self::print_indent(level + 2);
                        println!("None");
                    }
                }

                Self::print_indent(level + 1);
                println!("Condition");
                match &for_stmt.condition {
                    Some(expr) => Self::print_positioned_expression(expr, level + 2),
                    None => {
                        Self::print_indent(level + 2);
                        println!("None");
                    }
                }

                Self::print_indent(level + 1);
                println!("Increment");
                match &for_stmt.increment {
                    Some(stmt) => Self::print_positioned_statement(stmt, level + 2),
                    None => {
                        Self::print_indent(level + 2);
                        println!("None");
                    }
                }

                Self::print_indent(level + 1);
                println!("Body");
                for stmt in &for_stmt.body {
                    Self::print_positioned_statement(stmt, level + 2);
                }
            }

            PengStatement::Loop(body) => {
                Self::print_indent(level);
                println!("Loop");
                for stmt in body {
                    Self::print_positioned_statement(stmt, level + 1);
                }
            }

            PengStatement::Break => {
                Self::print_indent(level);
                println!("Break");
            }

            PengStatement::Continue => {
                Self::print_indent(level);
                println!("Continue");
            }

            PengStatement::Assign(assign) => {
                Self::print_assign_statement(assign, level);
            }

            PengStatement::Expression(expr) => {
                Self::print_indent(level);
                println!("ExpressionStatement");
                Self::print_positioned_expression(expr, level + 1);
            }
        }
    }

    fn print_declaration(decl: &PengDeclaration, level: usize) {
        match decl {
            PengDeclaration::Variable(var) => {
                Self::print_indent(level);
                print!("VariableDeclaration");
                Self::print_position(&var.position);
                println!();
                Self::print_variable_declaration(&var.value, level + 1);
            }

            PengDeclaration::As(declaration) => {
                Self::print_indent(level);
                print!("AsDeclaration");
                Self::print_position(&declaration.position);
                println!();
                Self::print_as_declaration(
                    &declaration.value,
                    level + 1,
                );
            }

            PengDeclaration::Function(func) => {
                Self::print_indent(level);
                print!("FunctionDeclaration");
                Self::print_position(&func.position);
                println!();
                Self::print_function_declaration(&func.value, level + 1);
            }

            PengDeclaration::Type(typ) => {
                Self::print_indent(level);
                print!("TypeDeclaration");
                Self::print_position(&typ.position);
                println!();
                Self::print_type_declaration(&typ.value, level + 1);
            }

            PengDeclaration::Module(module) => {
                Self::print_indent(level);
                print!("ModuleDeclaration");
                Self::print_position(&module.position);
                println!();
                Self::print_module_declaration(&module.value, level + 1);
            }

            PengDeclaration::Operation(operation) => {
                Self::print_indent(level);
                print!("OperationDeclaration");
                Self::print_position(&operation.position);
                println!();
                Self::print_operation_declaration(&operation.value, level + 1);
            }
        }
    }

    fn print_variable_declaration(var: &PengVariableDeclaration, level: usize) {
        Self::print_indent(level);
        print!("Name: {}", var.name.value);
        Self::print_position(&var.name.position);
        println!();

        Self::print_indent(level);
        println!("TypeHint");
        match &var.type_hint {
            Some(t) => Self::print_positioned_type_expression(t, level + 1),
            None => {
                Self::print_indent(level + 1);
                println!("None");
            }
        }

        Self::print_indent(level);
        println!("Value");
        match &var.value {
            Some(v) => Self::print_positioned_expression(v, level + 1),
            None => {
                Self::print_indent(level + 1);
                println!("None");
            }
        }
    }

    fn print_as_declaration(
        declaration: &PengAsDeclaration,
        level: usize,
    ) {
        Self::print_indent(level);
        print!("Name: {}", declaration.name.value);
        Self::print_position(&declaration.name.position);
        println!();

        Self::print_indent(level);
        println!("Value");
        Self::print_positioned_expression(
            &declaration.value,
            level + 1,
        );
    }

    fn print_function_declaration(func: &PengFunctionDeclaration, level: usize) {
        Self::print_indent(level);
        print!("Name: {}", func.name.value);
        Self::print_position(&func.name.position);
        println!();

        Self::print_generics(&func.generics, level);
        Self::print_function_params("Params", &func.params, level);
        Self::print_return_type(&func.return_type, level);

        Self::print_indent(level);
        println!("Body");
        for stmt in &func.body {
            Self::print_positioned_statement(stmt, level + 1);
        }
    }

    fn print_type_declaration(typ: &PengTypeDeclaration, level: usize) {
        Self::print_indent(level);
        print!("Name: {}", typ.name.value);
        Self::print_position(&typ.name.position);
        println!();

        Self::print_generics(&typ.generics, level);

        Self::print_indent(level);
        println!("Value");
        match &typ.value {
            Some(value) => Self::print_positioned_type_expression(value, level + 1),
            None => {
                Self::print_indent(level + 1);
                println!("None");
            }
        }

        Self::print_indent(level);
        println!("Supers");
        for sup in &typ.supers {
            Self::print_positioned_expression(sup, level + 1);
        }

        Self::print_indent(level);
        println!("Fields");
        for field in &typ.fields {
            Self::print_indent(level + 1);
            print!("Field");
            Self::print_position(&field.position);
            println!();
            Self::print_variable_declaration(&field.value, level + 2);
        }

        Self::print_indent(level);
        println!("Functions");
        for func in &typ.functions {
            Self::print_indent(level + 1);
            print!("Function");
            Self::print_position(&func.position);
            println!();
            Self::print_function_declaration(&func.value, level + 2);
        }
    }

    fn print_module_declaration(module: &PengModuleDeclaration, level: usize) {
        Self::print_indent(level);
        print!("Name: {}", module.name.value);
        Self::print_position(&module.name.position);
        println!();

        Self::print_indent(level);
        println!("Body");
        for decl in &module.body {
            Self::print_declaration(decl, level + 1);
        }
    }

    fn print_operation_declaration(operation: &PengOperationDeclaration, level: usize) {
        Self::print_indent(level);
        print!("Name: {}", operation.name.value);
        Self::print_position(&operation.name.position);
        println!();

        Self::print_function_params("Params", &operation.params, level);

        Self::print_indent(level);
        println!("Body");
        for stmt in &operation.body {
            Self::print_positioned_statement(stmt, level + 1);
        }
    }

    fn print_assign_statement(assign: &PengAssignStatement, level: usize) {
        Self::print_indent(level);

        match assign {
            PengAssignStatement::Assign { target, value } => {
                println!("Assign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::AddAssign { target, value } => {
                println!("AddAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::SubtractAssign { target, value } => {
                println!("SubtractAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::MultiplyAssign { target, value } => {
                println!("MultiplyAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::DivideAssign { target, value } => {
                println!("DivideAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::PowerAssign { target, value } => {
                println!("PowerAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
            PengAssignStatement::RemainderAssign { target, value } => {
                println!("RemainderAssign");
                Self::print_assign_parts(target, value, level + 1);
            }
        }
    }

    fn print_assign_parts(
        target: &PengPositionedExpression,
        value: &PengPositionedExpression,
        level: usize,
    ) {
        Self::print_indent(level);
        println!("Target");
        Self::print_positioned_expression(target, level + 1);

        Self::print_indent(level);
        println!("Value");
        Self::print_positioned_expression(value, level + 1);
    }

    fn print_positioned_expression(expr: &PengPositionedExpression, level: usize) {
        Self::print_indent(level);
        print!("Expression");
        Self::print_position(&expr.position);
        println!();

        Self::print_expression(&expr.value, level + 1);
    }

    fn print_expression(expr: &PengExpression, level: usize) {
        match expr {
            PengExpression::Literal(lit) => {
                Self::print_indent(level);
                print!("Literal");
                Self::print_position(&lit.position);
                println!();
                Self::print_literal(&lit.value, level + 1);
            }

            PengExpression::Identifier(id) => {
                Self::print_indent(level);
                print!("Identifier: {}", id.value);
                Self::print_position(&id.position);
                println!();
            }

            PengExpression::Type(typ) => {
                Self::print_indent(level);
                println!("TypeValue");
                Self::print_positioned_type_expression(typ, level + 1);
            }

            PengExpression::Unary { operator, value } => {
                Self::print_indent(level);
                println!("Unary: {:?}", operator);
                Self::print_indent(level + 1);
                println!("Value");
                Self::print_positioned_expression(value, level + 2);
            }

            PengExpression::Binary {
                left,
                operator,
                right,
            } => {
                Self::print_indent(level);
                println!("Binary: {:?}", operator);

                Self::print_indent(level + 1);
                println!("Left");
                Self::print_positioned_expression(left, level + 2);

                Self::print_indent(level + 1);
                println!("Right");
                Self::print_positioned_expression(right, level + 2);
            }

            PengExpression::FuncCall(call) => {
                Self::print_indent(level);
                println!("FuncCall");

                Self::print_indent(level + 1);
                println!("Function");
                Self::print_positioned_expression(&call.function, level + 2);

                Self::print_expression_list("Generics", &call.generics, level + 1);
                Self::print_expression_list("Args", &call.args, level + 1);
            }

            PengExpression::MethodCall(call) => {
                Self::print_indent(level);
                println!("MethodCall");

                Self::print_indent(level + 1);
                println!("Object");
                Self::print_positioned_expression(&call.object, level + 2);

                Self::print_indent(level + 1);
                print!("Method: {}", call.method.value);
                Self::print_position(&call.method.position);
                println!();

                Self::print_expression_list("Generics", &call.generics, level + 1);
                Self::print_expression_list("Args", &call.args, level + 1);
            }

            PengExpression::AttributeAccess(attr) => {
                Self::print_indent(level);
                println!("Attribute access");

                Self::print_indent(level + 1);
                println!("Object");
                Self::print_positioned_expression(&attr.object, level + 2);

                Self::print_indent(level + 1);
                print!("Name: {}", attr.name.value);
                Self::print_position(&attr.name.position);
                println!();
            }

            PengExpression::MemberAccess(mem) => {
                Self::print_indent(level);
                println!("member access");

                Self::print_indent(level + 1);
                println!("Object");
                Self::print_positioned_expression(&mem.object, level + 2);

                Self::print_indent(level + 1);
                print!("Name: {}", mem.name.value);
                Self::print_position(&mem.name.position);
                println!();
            }

            PengExpression::Index(index) => {
                Self::print_indent(level);
                println!("Index");

                Self::print_indent(level + 1);
                println!("Object");
                Self::print_positioned_expression(&index.object, level + 2);

                Self::print_indent(level + 1);
                println!("Index");
                Self::print_positioned_expression(&index.index, level + 2);
            }

            PengExpression::ObjectConstruction(obj) => {
                Self::print_indent(level);
                println!("ObjectConstruction");

                Self::print_indent(level + 1);
                println!("ObjectType");
                Self::print_positioned_expression(&obj.object_type, level + 2);

                Self::print_expression_list("Generics", &obj.generics, level + 1);

                Self::print_indent(level + 1);
                println!("Fields");
                for field in &obj.fields {
                    Self::print_object_field_literal(field, level + 2);
                }
            }

            PengExpression::OperationCall {
                left,
                operation,
                right,
            } => {
                Self::print_indent(level);
                println!("OperationCall");

                Self::print_indent(level + 1);
                println!("Left");
                Self::print_positioned_expression(left, level + 2);

                Self::print_indent(level + 1);
                println!("Operation");
                Self::print_positioned_expression(operation, level + 2);

                Self::print_indent(level + 1);
                println!("Right");
                Self::print_positioned_expression(right, level + 2);
            }

            PengExpression::Try { value, elsing } => {
                Self::print_indent(level);
                println!("Try");

                Self::print_indent(level + 1);
                println!("Value");
                Self::print_positioned_expression(value, level + 2);

                Self::print_indent(level + 1);
                println!("Else");
                match elsing {
                    Some(expr) => Self::print_positioned_expression(expr, level + 2),
                    None => {
                        Self::print_indent(level + 2);
                        println!("None");
                    }
                }
            }
        }
    }

    fn print_literal(lit: &PengLiteral, level: usize) {
        match lit {
            PengLiteral::Nil => {
                Self::print_indent(level);
                println!("Nil");
            }
            PengLiteral::Int(v) => {
                Self::print_indent(level);
                println!("Int: {}", v);
            }
            PengLiteral::Uint(v) => {
                Self::print_indent(level);
                println!("Uint: {}", v);
            }
            PengLiteral::Byte(v) => {
                Self::print_indent(level);
                println!("Byte: {}", v);
            }
            PengLiteral::Float32(v) => {
                Self::print_indent(level);
                println!("Float32: {}", v);
            }
            PengLiteral::Float64(v) => {
                Self::print_indent(level);
                println!("Float64: {}", v);
            }
            PengLiteral::Bool(v) => {
                Self::print_indent(level);
                println!("Bool: {}", v);
            }
            PengLiteral::String(v) => {
                Self::print_indent(level);
                println!("String: {:?}", v);
            }
            PengLiteral::Type(t) => {
                Self::print_indent(level);
                println!("TypeLiteral");
                Self::print_type_literal(t, level + 1);
            }
            PengLiteral::Function(f) => {
                Self::print_indent(level);
                println!("FunctionLiteral");
                Self::print_function_literal(f, level + 1);
            }
            PengLiteral::Module(m) => {
                Self::print_indent(level);
                println!("ModuleLiteral");
                Self::print_module_literal(m, level + 1);
            }
            PengLiteral::Vector(values) => {
                Self::print_indent(level);
                println!("Vector");
                for value in values {
                    Self::print_positioned_expression(value, level + 1);
                }
            }
            PengLiteral::Operation(op) => {
                Self::print_indent(level);
                println!("OperationLiteral");
                Self::print_operation_literal(op, level + 1);
            }
            PengLiteral::Object(fields) => {
                Self::print_indent(level);
                println!("ObjectLiteral");
                for field in fields {
                    Self::print_object_field_literal(field, level + 1);
                }
            }
        }
    }

    fn print_positioned_type_expression(typ: &PengPositionedTypeExpression, level: usize) {
        Self::print_indent(level);
        print!("TypeExpression");
        Self::print_position(&typ.position);
        println!();

        Self::print_type_expression(&typ.value, level + 1);
    }

    fn print_type_expression(typ: &PengTypeExpression, level: usize) {
        match typ {
            PengTypeExpression::Nil => {
                Self::print_indent(level);
                println!("Nil");
            }
            PengTypeExpression::Int => {
                Self::print_indent(level);
                println!("Int");
            }
            PengTypeExpression::Uint => {
                Self::print_indent(level);
                println!("Uint");
            }
            PengTypeExpression::Float32 => {
                Self::print_indent(level);
                println!("Float32");
            }
            PengTypeExpression::Float64 => {
                Self::print_indent(level);
                println!("Float64");
            }
            PengTypeExpression::Byte => {
                Self::print_indent(level);
                println!("Byte");
            }
            PengTypeExpression::Bool => {
                Self::print_indent(level);
                println!("Bool");
            }
            PengTypeExpression::String => {
                Self::print_indent(level);
                println!("String");
            }
            PengTypeExpression::Object => {
                Self::print_indent(level);
                println!("Object");
            }
            PengTypeExpression::Vector(inner) => {
                Self::print_indent(level);
                println!("Vector");
                Self::print_positioned_type_expression(inner, level + 1);
            }
            PengTypeExpression::Type => {
                Self::print_indent(level);
                println!("Type");
            }
            PengTypeExpression::Module => {
                Self::print_indent(level);
                println!("Module");
            }
            PengTypeExpression::Function => {
                Self::print_indent(level);
                println!("Function");
            }
            PengTypeExpression::Operation => {
                Self::print_indent(level);
                println!("Operation");
            }
            PengTypeExpression::Any => {
                Self::print_indent(level);
                println!("Any");
            }
            PengTypeExpression::Custom(expr) => {
                Self::print_indent(level);
                println!("Custom");
                Self::print_positioned_expression(expr, level + 1);
            }
            PengTypeExpression::UnionType => {
                Self::print_indent(level);
                println!("UnionType");
            }
            PengTypeExpression::Union(types) => {
                Self::print_indent(level);
                println!("Union");
                for typ in types {
                    Self::print_positioned_type_expression(typ, level + 1);
                }
            }
            PengTypeExpression::TypeLiteral(lit) => {
                Self::print_indent(level);
                println!("TypeLiteral");
                Self::print_type_literal(lit, level + 1);
            }
        }
    }

    fn print_type_literal(lit: &PengTypeLiteral, level: usize) {
        Self::print_generics(&lit.generics, level);

        Self::print_indent(level);
        println!("Supers");
        for sup in &lit.supers {
            Self::print_positioned_expression(sup, level + 1);
        }

        Self::print_indent(level);
        println!("Fields");
        for field in &lit.fields {
            Self::print_indent(level + 1);
            print!("Field");
            Self::print_position(&field.position);
            println!();
            Self::print_variable_declaration(&field.value, level + 2);
        }

        Self::print_indent(level);
        println!("Functions");
        for func in &lit.functions {
            Self::print_indent(level + 1);
            print!("Function");
            Self::print_position(&func.position);
            println!();
            Self::print_function_declaration(&func.value, level + 2);
        }
    }

    fn print_function_literal(lit: &PengFunctionLiteral, level: usize) {
        Self::print_generics(&lit.generics, level);
        Self::print_function_params("Params", &lit.params, level);
        Self::print_return_type(&lit.return_type, level);

        Self::print_indent(level);
        println!("Body");
        for stmt in &lit.body {
            Self::print_positioned_statement(stmt, level + 1);
        }
    }

    fn print_module_literal(lit: &PengModuleLiteral, level: usize) {
        Self::print_indent(level);
        println!("Body");
        for decl in &lit.body {
            Self::print_declaration(decl, level + 1);
        }
    }

    fn print_operation_literal(lit: &PengOperationLiteral, level: usize) {
        Self::print_function_params("Params", &lit.params, level);

        Self::print_indent(level);
        println!("Body");
        for stmt in &lit.body {
            Self::print_positioned_statement(stmt, level + 1);
        }
    }

    fn print_object_field_literal(field: &PengObjectFieldLiteral, level: usize) {
        Self::print_indent(level);
        print!("Field: {}", field.name.value);
        Self::print_position(&field.name.position);
        println!();

        Self::print_indent(level + 1);
        println!("Value");
        Self::print_positioned_expression(&field.value, level + 2);
    }

    fn print_generics(generics: &Vec<PengPositioned<String>>, level: usize) {
        Self::print_indent(level);
        println!("Generics");
        for generic in generics {
            Self::print_indent(level + 1);
            print!("{}", generic.value);
            Self::print_position(&generic.position);
            println!();
        }
    }

    fn print_function_params(
        title: &str,
        params: &Vec<PengPositionedFunctionParam>,
        level: usize,
    ) {
        Self::print_indent(level);
        println!("{}", title);

        for param in params {
            Self::print_indent(level + 1);
            print!("Param");
            Self::print_position(&param.position);
            println!();

            Self::print_indent(level + 2);
            print!("Name: {}", param.value.name.value);
            if param.value.variadic {
                print!("...");
            }
            Self::print_position(&param.value.name.position);
            println!();

            Self::print_indent(level + 2);
            println!("TypeHint");
            match &param.value.type_hint {
                Some(t) => Self::print_positioned_type_expression(t, level + 3),
                None => {
                    Self::print_indent(level + 3);
                    println!("None");
                }
            }
        }
    }

    fn print_return_type(
        return_type: &Option<PengPositionedTypeExpression>,
        level: usize,
    ) {
        Self::print_indent(level);
        println!("ReturnType");

        match return_type {
            Some(t) => Self::print_positioned_type_expression(t, level + 1),
            None => {
                Self::print_indent(level + 1);
                println!("None");
            }
        }
    }

    fn print_expression_list(
        title: &str,
        values: &Vec<PengPositionedExpression>,
        level: usize,
    ) {
        Self::print_indent(level);
        println!("{}", title);

        for value in values {
            Self::print_positioned_expression(value, level + 1);
        }
    }
}


#[derive(Debug, Clone)]
pub enum PengDeclaration {
    Variable(PengPositionedVariableDeclaration),
    As(PengPositionedAsDeclaration),
    Function(PengPositionedFunctionDeclaration),
    Type(PengPositionedTypeDeclaration),
    Module(PengPositionedModuleDeclaration),
    Operation(PengPositionedOperationDeclaration),
}

#[derive(Debug, Clone)]
pub enum PengStatement {
    Block(Vec<PengPositionedStatement>),

    Declaration(PengDeclaration),

    Return(Option<PengPositionedExpression>),

    If(PengIfStatement),
    While(PengWhileStatement),
    For(PengForStatement),
    Loop(Vec<PengPositionedStatement>),

    Break,
    Continue,

    Assign(PengAssignStatement),
    Expression(PengPositionedExpression),
}

#[derive(Debug, Clone)]
pub enum PengAssignStatement {
    Assign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    AddAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    SubtractAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    MultiplyAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    DivideAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    PowerAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
    RemainderAssign {
        target: PengPositionedExpression,
        value: PengPositionedExpression,
    },
}

#[derive(Debug, Clone)]
pub enum PengExpression {
    Literal(PengPositionedLiteral),

    Identifier(PengPositioned<String>),
    Type(PengPositionedTypeExpression),

    Unary {
        operator: PengUnaryOperator,
        value: Box<PengPositionedExpression>,
    },

    Binary {
        left: Box<PengPositionedExpression>,
        operator: PengBinaryOperator,
        right: Box<PengPositionedExpression>,
    },

    FuncCall(PengFuncCallExpression),
    MethodCall(PengMethodCallExpression),

    AttributeAccess(PengAttributeAccessExpression), // dot access
    MemberAccess(PengMemberAccessExpression),   // colon access
    Index(PengIndexExpression),

    ObjectConstruction(PengObjectConstructionExpression),

    OperationCall {
        left: Box<PengPositionedExpression>,
        operation: Box<PengPositionedExpression>,
        right: Box<PengPositionedExpression>,
    },

    Try {
        value: Box<PengPositionedExpression>,
        elsing: Option<Box<PengPositionedExpression>>,
    },
}

#[derive(Debug, Clone)]
pub enum PengLiteral {
    Nil,

    Int(isize),
    Uint(usize),
    Byte(u8),

    Float32(f32),
    Float64(f64),

    Bool(bool),
    String(String),

    Type(PengTypeLiteral),
    Function(PengFunctionLiteral),
    Module(PengModuleLiteral),
    Vector(Vec<PengPositionedExpression>),
    Operation(PengOperationLiteral),
    Object(Vec<PengObjectFieldLiteral>),
}

#[derive(Debug, Clone)]
pub struct PengVariableDeclaration {
    pub name: PengPositioned<String>,
    pub type_hint: Option<PengPositionedTypeExpression>,
    pub value: Option<PengPositionedExpression>,
}

#[derive(Debug, Clone)]
pub struct PengAsDeclaration {
    pub name: PengPositioned<String>,
    pub value: PengPositionedExpression,
}

#[derive(Debug, Clone)]
pub struct PengFunctionDeclaration {
    pub name: PengPositioned<String>,
    pub generics: Vec<PengPositioned<String>>,
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengFunctionParam {
    pub name: PengPositioned<String>,
    pub type_hint: Option<PengPositionedTypeExpression>,
    pub variadic: bool,
}

#[derive(Debug, Clone)]
pub struct PengTypeDeclaration {
    pub name: PengPositioned<String>,
    pub generics: Vec<PengPositioned<String>>,
    pub value: Option<PengPositionedTypeExpression>,
    pub supers: Vec<PengPositionedExpression>,
    pub fields: Vec<PengPositionedVariableDeclaration>,
    pub functions: Vec<PengPositionedFunctionDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengModuleDeclaration {
    pub name: PengPositioned<String>,
    pub body: Vec<PengDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengOperationDeclaration {
    pub name: PengPositioned<String>,
    pub params: Vec<PengPositionedFunctionParam>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengIfStatement {
    pub condition: PengPositionedExpression,
    pub then_branch: Vec<PengPositionedStatement>,
    pub else_branch: Option<Vec<PengPositionedStatement>>,
}

#[derive(Debug, Clone)]
pub struct PengWhileStatement {
    pub condition: PengPositionedExpression,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengForStatement {
    pub initializer: Option<Box<PengPositionedStatement>>,
    pub condition: Option<PengPositionedExpression>,
    pub increment: Option<Box<PengPositionedStatement>>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengFuncCallExpression {
    pub function: Box<PengPositionedExpression>,
    pub generics: Vec<PengPositionedExpression>,
    pub args: Vec<PengPositionedExpression>,
}

#[derive(Debug, Clone)]
pub struct PengMethodCallExpression {
    pub object: Box<PengPositionedExpression>,
    pub method: PengPositioned<String>,
    pub generics: Vec<PengPositionedExpression>,
    pub args: Vec<PengPositionedExpression>,
}

#[derive(Debug, Clone)]
pub struct PengAttributeAccessExpression {
    pub object: Box<PengPositionedExpression>,
    pub name: PengPositioned<String>,
}

#[derive(Debug, Clone)]
pub struct PengMemberAccessExpression {
    pub object: Box<PengPositionedExpression>,
    pub name: PengPositioned<String>,
}

#[derive(Debug, Clone)]
pub struct PengIndexExpression {
    pub object: Box<PengPositionedExpression>,
    pub index: Box<PengPositionedExpression>,
}

#[derive(Debug, Clone)]
pub struct PengObjectConstructionExpression {
    pub object_type: Box<PengPositionedExpression>,
    pub generics: Vec<PengPositionedExpression>,
    pub fields: Vec<PengObjectFieldLiteral>,
}

#[derive(Debug, Clone)]
pub enum PengUnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum PengBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Remainder,

    Concat,

    ShortCircuitAnd,
    ShortCircuitOr,
    NonShortCircuitAnd,
    NonShortCircuitOr,

    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualsThan,
    LessThan,
    LessEqualsThan,

    As,
}

#[derive(Debug, Clone)]
pub enum PengTypeExpression {
    Nil,
    Int,
    Uint,
    Float32,
    Float64,
    Byte,
    Bool,
    String,

    Object,
    Vector(Box<PengPositionedTypeExpression>),

    Type,
    Module,
    Function,
    Operation,
    Any,

    Custom(Box<PengPositionedExpression>),
    TypeLiteral(PengTypeLiteral),

    UnionType,
    Union(Vec<PengPositionedTypeExpression>),
}

#[derive(Debug, Clone)]
pub struct PengTypeLiteral {
    pub generics: Vec<PengPositioned<String>>,
    pub supers: Vec<PengPositionedExpression>,
    pub fields: Vec<PengPositionedVariableDeclaration>,
    pub functions: Vec<PengPositionedFunctionDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengFunctionLiteral {
    pub generics: Vec<PengPositioned<String>>,
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengModuleLiteral {
    pub body: Vec<PengDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengOperationLiteral {
    pub params: Vec<PengPositionedFunctionParam>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengObjectFieldLiteral {
    pub name: PengPositioned<String>,
    pub value: PengPositionedExpression,
}



pub fn parse(ptokens: Vec<PengPositionedToken>) -> Result<PengAST, PengError> {
    match parse_script(ptokens) {
        Ok(ast) => Ok(ast),
        Err(e) => Err(e),
    }
}
