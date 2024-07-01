use crate::error::{runtime_error, LoxError};
use crate::object::Object;
use crate::scanner;

#[derive(Debug)]
pub enum Unwinder<'ast, 'src: 'ast> {
    Return {
        keyword: &'ast scanner::Token<'src>,
        value: Object<'ast, 'src>,
    },
    Err(LoxError),
}

impl<'ast, 'src: 'ast> Unwinder<'ast, 'src> {
    pub fn err<T>(token: &scanner::Token, message: &str) -> Result<T, Self> {
        Err(Self::Err(runtime_error(token, message)))
    }

    pub fn promote<T>(result: Result<T, LoxError>) -> Result<T, Self> {
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(Self::Err(e)),
        }
    }
}
