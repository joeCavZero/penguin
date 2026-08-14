use crate::core::*;
use crate::lexer::*;
use std::iter::Peekable;
use std::slice::Iter;

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

pub type PengPositionedFunctionCallArg = PengPositioned<PengFunctionCallArg>;

#[derive(Debug, Clone)]
pub enum PengAST {
    Program(Vec<PengBindedDeclaration>),
    Script(Vec<PengPositionedStatement>),
}

#[derive(Debug, Clone)]
pub enum PengDeclaration {
    Var(PengPositionedVariableDeclaration),
    As(PengPositionedAsDeclaration),
    Function(PengPositionedFunctionDeclaration),
    Type(PengPositionedTypeDeclaration),
    Module(PengPositionedModuleDeclaration),
    Operation(PengPositionedOperationDeclaration),
}

pub type PengBindedDeclaration = PengBinded<PengDeclaration>;

#[derive(Debug, Clone)]
pub enum PengStatement {
    Block(Vec<PengPositionedStatement>),

    Declaration(PengBindedDeclaration),

    Return(Option<PengPositionedExpression>),

    If(PengIfStatement),
    Match(PengMatchStatement),
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
    MemberAccess(PengMemberAccessExpression),       // colon access
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
    pub type_hint: Option<PengPositionedTypeExpression>,
    pub value: PengPositionedExpression,
}

#[derive(Debug, Clone)]
pub struct PengFunctionDeclaration {
    pub name: PengPositioned<String>,
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
pub struct PengFunctionCallArg {
    pub expression: PengPositionedExpression,
    pub variadic: bool,
}

#[derive(Debug, Clone)]
pub struct PengTypeDeclaration {
    pub name: PengPositioned<String>,
    pub value: Option<PengPositionedTypeExpression>,
    pub supers: Vec<PengPositionedExpression>,
    pub fields: Vec<PengBinded<PengPositionedVariableDeclaration>>,
    pub functions: Vec<PengBinded<PengPositionedFunctionDeclaration>>,
}

#[derive(Debug, Clone)]
pub struct PengModuleDeclaration {
    pub name: PengPositioned<String>,
    pub body: Vec<PengBindedDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengOperationDeclaration {
    pub name: PengPositioned<String>,
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengIfStatement {
    pub condition: PengPositionedExpression,
    pub then_branch: Vec<PengPositionedStatement>,
    pub else_branch: Option<Vec<PengPositionedStatement>>,
}

#[derive(Debug, Clone)]
pub struct PengMatchStatement {
    pub value: PengPositionedExpression,
    pub arms: Vec<PengMatchArm>,
    pub elsing: Option<Vec<PengPositionedStatement>>,
}

#[derive(Debug, Clone)]
pub struct PengMatchArm {
    pub pattern: PengPositionedExpression,
    pub body: Vec<PengPositionedStatement>,
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
    pub args: Vec<PengPositionedFunctionCallArg>,
}

#[derive(Debug, Clone)]
pub struct PengMethodCallExpression {
    pub object: Box<PengPositionedExpression>,
    pub method: PengPositioned<String>,
    pub args: Vec<PengPositionedFunctionCallArg>,
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
    Thread,
    Any,

    Custom(Box<PengPositionedExpression>),
    TypeLiteral(PengTypeLiteral),
}

#[derive(Debug, Clone)]
pub struct PengTypeLiteral {
    pub supers: Vec<PengPositionedExpression>,
    pub fields: Vec<PengBinded<PengPositionedVariableDeclaration>>,
    pub functions: Vec<PengBinded<PengPositionedFunctionDeclaration>>,
}

#[derive(Debug, Clone)]
pub struct PengFunctionLiteral {
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengModuleLiteral {
    pub body: Vec<PengBindedDeclaration>,
}

#[derive(Debug, Clone)]
pub struct PengOperationLiteral {
    pub params: Vec<PengPositionedFunctionParam>,
    pub return_type: Option<PengPositionedTypeExpression>,
    pub body: Vec<PengPositionedStatement>,
}

#[derive(Debug, Clone)]
pub struct PengObjectFieldLiteral {
    pub name: PengPositioned<String>,
    pub value: PengPositionedExpression,
}
