use crate::core::*;

pub type PengPositionedAST = PengPositioned<PengAST>;
pub type PengPositionedStatement = PengPositioned<PengStatement>;
pub type PengPositionedExpression = PengPositioned<PengExpression>;
pub type PengPositionedTypeExpression = PengPositioned<PengTypeExpression>;

pub type PengPositionedVariableDeclaration = PengPositioned<PengVariableDeclaration>;
pub type PengPositionedFunctionDeclaration = PengPositioned<PengFunctionDeclaration>;
pub type PengPositionedTypeDeclaration = PengPositioned<PengTypeDeclaration>;
pub type PengPositionedModuleDeclaration = PengPositioned<PengModuleDeclaration>;
pub type PengPositionedOperationDeclaration = PengPositioned<PengOperationDeclaration>;

pub type PengPositionedFunctionParam = PengPositioned<PengFunctionParam>;

pub type PengPositionedLiteral = PengPositioned<PengLiteral>;

#[derive(Debug, Clone)]
pub struct PengAST {
    pub program: Vec<PengPositionedStatement>,
}


impl PengAST {
    pub fn pretty_print(&self) {
        println!("AST program:");
        for stmt in &self.program {
            self.print_statement(&stmt.value, 1);
        }
    }

    fn print_indent(level: usize) {
        print!("{}", "  ".repeat(level));
    }

    fn print_statement(&self, stmt: &PengStatement, level: usize) {
        match stmt {
            PengStatement::Block(stmts) => {
                Self::print_indent(level); println!("Block");
                for s in stmts { self.print_statement(&s.value, level + 1); }
            }
            PengStatement::Declaration(decl) => self.print_declaration(decl, level),
            PengStatement::Return(expr) => {
                Self::print_indent(level); println!("Return");
                if let Some(e) = expr { self.print_expression(&e.value, level + 1); }
            }
            PengStatement::If(if_stmt) => {
                Self::print_indent(level); println!("If Statement");
                self.print_expression(&if_stmt.condition.value, level + 1);
                for s in &if_stmt.then_branch { self.print_statement(&s.value, level + 2); }
            }
            PengStatement::Expression(expr) => self.print_expression(&expr.value, level),
            _ => {
                Self::print_indent(level);
                println!("Statement: {:?}", stmt);
            }
        }
    }

    fn print_declaration(&self, decl: &PengDeclaration, level: usize) {
        Self::print_indent(level);
        match decl {
            PengDeclaration::Variable(v) => {
                println!("VarDecl: {}", v.value.name.value);
                if let Some(val) = &v.value.value {
                    self.print_expression(&val.value, level + 1);
                }
            }
            PengDeclaration::Function(f) => {
                println!("FuncDecl: {}", f.value.name.value);
                for s in &f.value.body { self.print_statement(&s.value, level + 1); }
            }
            PengDeclaration::Module(m) => {
                println!("Module: {}", m.value.name.value);
                for d in &m.value.body { self.print_declaration(d, level + 1); }
            }
            _ => println!("Declaration: {:?}", decl),
        }
    }

    fn print_expression(&self, expr: &PengExpression, level: usize) {
        Self::print_indent(level);
        match expr {
            PengExpression::Literal(lit) => println!("Literal: {:?}", lit.value),
            PengExpression::Identifier(id) => println!("Identifier: {}", id.value),
            PengExpression::Binary { left, operator, right } => {
                println!("BinaryOp: {:?}", operator);
                self.print_expression(&left.value, level + 1);
                self.print_expression(&right.value, level + 1);
            }
            PengExpression::Unary { operator, value } => {
                println!("UnaryOp: {:?}", operator);
                self.print_expression(&value.value, level + 1);
            }
            PengExpression::FuncCall(f) => {
                println!("FuncCall: {:?}", f.function.value);
            }
            _ => println!("Expression: {:?}", expr),
        }
    }
}


#[derive(Debug, Clone)]
pub enum PengDeclaration {
    Variable(PengPositionedVariableDeclaration),
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

    Break,
    Continue,

    Expression(PengPositionedExpression),
}

#[derive(Debug, Clone)]
pub enum PengExpression {
    Literal(PengPositionedLiteral),

    Identifier(PengPositioned<String>),

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

    Attribute(PengAttributeExpression),
    Index(PengIndexExpression),
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
pub struct PengFunctionDeclaration {
    pub name: PengPositioned<String>,
    pub generics: Vec<PengPositioned<String>>,
    pub env_params: Vec<PengPositionedFunctionParam>,
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengFunctionParam {
    pub name: PengPositioned<String>,
    pub type_hint: Option<PengPositionedTypeExpression>,
}

#[derive(Debug, Clone)]
pub struct PengTypeDeclaration {
    pub name: PengPositioned<String>,
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
pub struct PengAttributeExpression {
    pub object: Box<PengPositionedExpression>,
    pub name: PengPositioned<String>,
}

#[derive(Debug, Clone)]
pub struct PengIndexExpression {
    pub object: Box<PengPositionedExpression>,
    pub index: Box<PengPositionedExpression>,
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

    And,
    Or,

    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualsThan,
    LessThan,
    LessEqualsThan,

    Is,
    As,

    Assign,

    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    PowerAssign,
    RemainderAssign,
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
    Operator,
    Any,

    Named(PengPositioned<String>),

    Union(Vec<PengPositionedTypeExpression>),
    Nillable(Box<PengPositionedTypeExpression>),
}

#[derive(Debug, Clone)]
pub struct PengTypeLiteral {
    pub supers: Vec<PengPositionedExpression>,
    pub fields: Vec<PengPositionedVariableDeclaration>,
    pub functions: Vec<PengPositionedFunctionDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengFunctionLiteral {
    pub generics: Vec<PengPositioned<String>>,
    pub env_params: Vec<PengPositionedFunctionParam>,
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
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengObjectFieldLiteral {
    pub name: PengPositioned<String>,
    pub value: PengPositionedExpression,
}