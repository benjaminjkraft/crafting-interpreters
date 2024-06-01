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
