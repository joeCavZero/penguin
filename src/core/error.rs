use crate::position::*;

#[derive(Debug, Clone)]
pub enum PengError {
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
}