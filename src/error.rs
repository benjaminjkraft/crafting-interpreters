use crate::scanner;
use std::fmt;

#[derive(Debug)]
pub struct LoxError {
    pub line: usize,
    pub loc: String,
    pub message: String,
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, self.loc, self.message
        )
    }
}

pub fn err(token: scanner::Token, message: &str) -> LoxError {
    let loc = if token.type_ == scanner::TokenType::EOF {
        " at end".to_string()
    } else {
        format!(" at '{}'", token.lexeme)
    };
    LoxError {
        line: token.line,
        loc,
        message: message.to_string(),
    }
}
