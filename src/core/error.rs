use crate::core::cell::*;
use crate::core::instruction::*;
use crate::core::position::*;
use crate::core::typing::*;
use crate::core::utils::*;
use crate::core::value::*;
use crate::lexer::token::*;

#[derive(Debug, Clone)]
pub enum PengError {
    Stack(Vec<PengError>),

    Position(PengPosition),

    PositionedMessage {
        message: String,
        position: PengPosition,
    },

    PositionedError {
        error: Box<PengError>,
        position: PengPosition,
    },

    // ======================
    // Erros gerais
    // ======================
    InternalError(String),

    NotImplemented(String),

    InvalidState(String),

    // ======================
    // Erros de nome / escopo
    // ======================
    NameNotFound(PengNamePoolPtr),

    NameAlreadyDefined(PengNamePoolPtr),

    LocalNotFound(usize),

    GlobalNotFound(PengNamePoolPtr),

    AttributeNotFound(PengNamePoolPtr),

    AttributeAlreadyDefined(PengNamePoolPtr),

    // ======================
    // Erros de heap / referência
    // ======================
    HeapValueNotFound(PengHeapPtr),

    InvalidReference(PengHeapPtr),

    ExpectedReference,

    DanglingReference(PengHeapPtr),

    // ======================
    // Erros de mutabilidade / estado
    // ======================
    CannotAssignImmutable,

    CannotMutateImmutable,

    CannotReadUninitialized,

    CannotAssignUninitialized,

    ExpectedInitialized,

    ExpectedMutable,

    ExpectedImmutable,

    // ======================
    // Erros de tipo
    // ======================
    TypeMismatch {
        expected: String,
        found: String,
    },

    InvalidConversion {
        from: PengValue,
        to: PengType,
    },

    CannotInferType,

    ExpectedType,

    ExpectedValue,

    ExpectedFunction,

    ExpectedThread,

    ExpectedObject,

    ExpectedVector,

    ExpectedModule,

    ExpectedOperation,

    ExpectedUnion,

    ExpectedNumber,

    // ======================
    // Erros de operação
    // ======================
    InvalidUnaryOperation {
        operator: String,
        operand: String,
    },

    InvalidBinaryOperationCell {
        operator: String,
        left: PengCell,
        right: PengCell,
    },

    InvalidBinaryOperationValue {
        operator: String,
        left: PengValue,
        right: PengValue,
    },

    DivisionByZero,

    ArithmeticOverflow,

    ArithmeticUnderflow,

    NegativeUnsignedResult,

    // ======================
    // Erros de índice
    // ======================
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },

    InvalidIndexTypeValue(PengValue),

    CannotIndexValue(String),

    CannotSetIndex(String),

    // ======================
    // Erros de chamada
    // ======================
    WrongArgumentCount {
        expected: usize,
        found: usize,
    },

    TooManyArguments {
        expected: usize,
        found: usize,
    },

    CannotCallValue(String),

    ReturnOutsideFunction,

    MissingReturnValue,

    // ======================
    // Erros de stack / frame / runtime
    // ======================
    StackUnderflow,

    StackOverflow,

    EmptyStack,

    FrameNotFound,

    EmptyFrameStack,

    InvalidFrame,

    ProgramCounterOutOfBounds {
        pc: usize,
        len: usize,
    },

    InstructionExpectedConstant,

    InvalidInstruction(PengInstruction),

    ThreadNotFound(PengHeapPtr),

    CurrentThreadNotFound,

    // ======================
    // Erros de parsing / compilação
    // ======================
    SyntaxError(String),

    UnexpectedToken(PengToken),

    ExpectedToken {
        expected: String,
        found: String,
    },

    InvalidAssignmentTarget,

    InvalidLValue,

    BreakOutsideLoop,

    ContinueOutsideLoop,

    RaiseOutsideTry,

    // ======================
    // Erros de usuário
    // ======================
    Raised(Box<PengError>),
}

impl PengError {
    pub fn push(self, e: PengError) -> Self {
        match self {
            Self::Stack(mut s) => {
                s.push(e);
                return Self::Stack(s);
            }
            _ => {
                return Self::Stack(vec![self, e]);
            }
        }
    }

    pub fn new_position(position: PengPosition) -> Self {
        Self::Position(position)
    }

    pub fn new_positioned_message(message: String, position: PengPosition) -> Self {
        Self::PositionedMessage {
            message: message,
            position: position,
        }
    }

    pub fn new_positioned_error(error: PengError, position: PengPosition) -> Self {
        Self::PositionedError {
            error: Box::new(error),
            position: position,
        }
    }

    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Stack(left), Self::Stack(right)) => {
                left.len() == right.len() && left.iter().zip(right.iter()).all(|(a, b)| a.equals(b))
            }

            (Self::Position(left), Self::Position(right)) => left.equals(right),

            (
                Self::PositionedMessage {
                    message: left_message,
                    position: left_position,
                },
                Self::PositionedMessage {
                    message: right_message,
                    position: right_position,
                },
            ) => left_message == right_message && left_position.equals(right_position),

            (
                Self::PositionedError {
                    error: left_error,
                    position: left_position,
                },
                Self::PositionedError {
                    error: right_error,
                    position: right_position,
                },
            ) => left_error.equals(right_error) && left_position.equals(right_position),

            _ => false,
        }
    }
}
