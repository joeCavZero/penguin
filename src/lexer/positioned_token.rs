use crate::core::*;
use crate::lexer::*;

pub type PengPositionedToken = PengPositioned<PengToken>;

impl PengPositioned<PengToken> {
    pub fn from_string(source: String, position: PengPosition) -> Result<Self, PengError> {
        match PengToken::from_string(source) {
            Ok(token) => Ok(Self {
                value: token,
                position,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn new_string(source: String, position: PengPosition) -> Self {
        Self {
            value: PengToken::new_string(source),
            position,
        }
    }

    pub fn token_display(&self) -> String {
        match &self.value {
            PengToken::Identifier(s) => format!("Identifier({})", s),
            PengToken::StringLiteral(s) => format!("String(\"{}\")", s),
            other => format!("{:?}", other),
        }
    }
}