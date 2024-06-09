use crate::scanner;
use std::fmt;

#[derive(Debug)]
pub struct LoxError {
    pub line: usize,
    pub loc: String,
    pub exit: u8,
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

pub fn parse_error(token: scanner::Token, message: &str) -> LoxError {
    let loc = if token.type_ == scanner::TokenType::EOF {
        " at end".to_string()
    } else {
        format!(" at '{}'", token.lexeme)
    };
    LoxError {
        line: token.line,
        loc,
        exit: 65,
        message: message.to_string(),
    }
}

pub fn runtime_error<T>(token: &scanner::Token, message: &str) -> Result<T, LoxError> {
    Err(LoxError {
        line: token.line,
        loc: "".to_string(),
        exit: 70,
        message: message.to_string(),
    })
}
