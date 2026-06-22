use crate::position::*;

#[derive(Debug, Clone)]
pub enum PengError {
    Code(PengErrorCode),
    Message(String),
    Position(PengPosition),
    PositionedMessage {
        message: String,
        position: PengPosition,
    },
}

impl PengError {
    pub fn new_message(message: String) -> Self {
        Self::Message(message)
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

    pub fn equals(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Message(left), Self::Message(right)) => left == right,
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
            ) => {
                left_message == right_message
                    && left_position.equals(right_position)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PengErrorCode {
    TestError,
}