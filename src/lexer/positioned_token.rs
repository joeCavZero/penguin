use crate::core::*;
use crate::lexer::*;

#[derive(Clone, Debug)]
pub struct PengPositionedToken {
    pub token: PengToken,
    pub position: PengPosition,
}

impl PengPositionedToken {
    pub fn from_string(source: String, position: PengPosition) -> Result<Self, String> {
        match PengToken::from_string(source) {
            Ok(tkn) => {
                Ok(
                    Self {
                        token: tkn,
                        position: position,
                    }
                )
            }
            Err(e) => Err(e)
        } 
    }

    pub fn new_string(source: String, position: PengPosition) -> Self {
        Self {
            token: PengToken::new_string(source),
            position: position,
        }
    }

    pub fn token_display(&self) -> String {
        match &self.token {
            PengToken::Identifier(s) => format!("Identifier({})", s),
            PengToken::NumberLiteral(v) => format!("NumberLiteral({})", v),
            PengToken::StringLiteral(s) => format!("String(\"{}\")", s),
            other => format!("{:?}", other), // fallback seguro
        }
    }

}